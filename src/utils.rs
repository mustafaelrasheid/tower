use std::error::Error;
use std::fs::{read_to_string, read_dir, write, remove_file, remove_dir};
use std::io::Read;
use std::io::Error as IOError;
use std::path::Path;
use std::collections::HashMap;
use xz2::read::XzDecoder;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Archive as TarArchive;
use tar::Builder as TarBuilder;
use tar::Header as TarHeader;
use ar::Archive as ArArchive;
use serde_json::{Value};
use crate::error::ArchiveError;
use crate::atom::Atom;
use crate::lock::{Lock, DirectoryEntry};

pub fn read_file_as_json(path: &str)
-> Result<Value, Box<dyn Error>> {
    let file = read_to_string(&path)?;
    let json: Value = serde_json::from_str(&file)?;

    return Ok(json); 
}

pub fn read_collection_as_json(path: &str)
-> Result<Vec<Value>, Box<dyn Error>> {
    let mut collection = Vec::new();
    
    for entry in read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = read_to_string(&path)?;
            let item: Value = serde_json::from_str(&content)?;
            collection.push(item);
        }
    }
    
    return Ok(collection);
}

pub fn write_file_as_json(path: &str, value: &Value)
-> Result<(), IOError> {
    return write(&path, serde_json::to_string_pretty(&value).unwrap());
}

pub fn map_atom_to_entries(
    lock: &Lock,
    path: &str,
    list_dirs: bool,
    ignores: &[&str]
) -> Vec<String> {

    let mut entries: Vec<String> = Vec::new();
    
    match lock {
        Lock::File(file) => {
            let is_ignore = if let Some(modif) = &file.modification {
                ignores.contains(&modif.as_ref())
            } else { false };
            
            if !is_ignore {
                entries.push(path.to_string());
            }
        },
        Lock::Dir(dir) => {
            if list_dirs {
                entries.push(path.to_string());
            }
            for (name, value) in &dir.contents {
                let filesystem_path = format!("{}/{}", path, name);
                let mut nested = map_atom_to_entries(
                    value,
                    &filesystem_path,
                    list_dirs,
                    ignores
                );
                entries.append(&mut nested);
            }
        }
    }
    

    return entries;
}

pub fn map_entries_to_atom(
    path: &str,
    lock: &Lock,
    ignore: &Lock,
    recusrive: bool
) -> Result<Vec<String>, IOError> {
    
    let mut output: Vec<String> = Vec::new();
    let lock   = if let Lock::Dir(dir) = lock   { dir } else { return Ok(output); };
    let ignore = if let Lock::Dir(dir) = ignore { dir } else { return Ok(output); };
    let read_dir_path = if path == "" { "/" } else { path };
    let entries = read_dir(read_dir_path)?;
            
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        let full_path = format!("{}/{}", path, name_str);
        let is_found = lock.contents.contains_key(&name_str);
        let ignoreable = ignore.contents.contains_key(&name_str);
        
        if ignoreable {
            continue;
        }
        
        if !is_found {
            output.push(full_path.clone());
        }

        if !entry.file_type()?.is_dir() || (!recusrive && !is_found) {
            continue;
        }
        if !recusrive{
            if !is_found {
                continue;
            }
        }

        let empty_lock = Lock::Dir(DirectoryEntry {
            count: None,
            contents: HashMap::new(),
        });

        let mut nested = map_entries_to_atom(
            &full_path,
            &lock.contents.get(&name_str).unwrap_or(&empty_lock),
            &empty_lock,
            recusrive
        )?;
        output.append(&mut nested);
    }

    return Ok(output);
}

pub fn decompress_package(data: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    if data.starts_with(b"\x1f\x8b") {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| ArchiveError::Compression(e))?;
        
        return Ok(decompressed);
    }
    if data.starts_with(b"\xfd7zXZ\x00") {
        let mut decoder = XzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| ArchiveError::Compression(e))?;
        
        return Ok(decompressed);
    }
    
    return Ok(data.to_vec());
}

fn extract_tar(data: &[u8]) -> Result<Vec<(String, u32, Vec<u8>)>, ArchiveError> {
    let mut archive = TarArchive::new(data);
    let mut files = Vec::new();
    
    for entry_result in archive.entries()
        .map_err(|e| ArchiveError::Archive(e))? {
        let mut entry = entry_result
            .map_err(|e| ArchiveError::Archive(e))?;
        
        if !entry.header().entry_type().is_file() {
            continue;
        }
        
        let mut buffer = Vec::new();
        let path = entry.path()
            .map_err(|e| ArchiveError::Archive(e))?
            .to_string_lossy()
            .to_string();
        let permission = entry.header().mode().map_err(|e| ArchiveError::Archive(e))?;

        entry.read_to_end(&mut buffer)
            .map_err(|e| ArchiveError::Archive(e))?;                
        files.push((path, permission, buffer));
    }
    
    return Ok(files);
}

fn extract_ar(data: &[u8])
-> Result<Vec<(String, u32, Vec<u8>)>, ArchiveError> {
    let mut ar = ArArchive::new(data);
    let mut files = Vec::new();
    
    while let Some(entry_result) = ar.next_entry() {
        let mut buffer = Vec::new();
        let mut entry = entry_result
            .map_err(|e| ArchiveError::Archive(e))?;
        let name = String::from_utf8_lossy(entry.header().identifier())
            .to_string();
        let permission = entry.header().mode();
        
        entry.read_to_end(&mut buffer)
            .map_err(|e| ArchiveError::Archive(e))?;
        files.push((name, permission, buffer));
    }
    
    return Ok(files);
}

pub fn uncover_archive(archive: &[u8])
-> Result<Vec<(String, u32, Vec<u8>)>, ArchiveError> {
    let archive = decompress_package(&archive)?;

    if archive.starts_with(b"!<arch>") || archive.starts_with(b"!<thin>") {
        return Ok(extract_ar(&archive)?);
    }
    if archive.len() > 262 && &archive[257..262] == b"ustar" {
        return Ok(extract_tar(&archive)?);
    }
    
    return Err(
        ArchiveError::FormatSupport("No valid archive type".to_string())
    );
}

pub fn create_package(atom: Atom)
-> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut tar = TarBuilder::new(
        GzEncoder::new(
            &mut buffer,
            Compression::default()
        ));
    let mut header = TarHeader::new_gnu();
    let metadata_string = serde_json::to_string_pretty(
            &atom.metadata
        ).unwrap();
    
    header.set_size(metadata_string.len() as u64);
    header.set_mode(0o644);
    tar.append_data(
        &mut header,
        "metadata.json",
        metadata_string.as_str().as_bytes()
    ).expect("Failed to append metadata to tar archive");
    
    for (path, perm, data) in atom.files {
        let mut header = TarHeader::new_gnu();

        header.set_size(data.len() as u64);
        header.set_mode(perm);
        tar.append_data(
            &mut header,
            &format!("contents/{}", path),
            &data[..]
        ).expect(
            &format!("Failed to append {} to tar archive", path)
        );
    }
    
    tar.finish().expect("Failed to build tar archive");
    
    drop(tar);
    return buffer;
}

pub fn parse_control(content: &str) -> Vec<(String, String)> {
    let mut fields: Vec<(String, String)> = Vec::new();

    for line in content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, value)) = fields.last_mut() {
                value.push('\n');
                value.push_str(line.trim());
            }
        } else if let Some(colon_pos) = line.find(':') {
            let field = line[..colon_pos].to_string();
            let value = line[colon_pos+1..].trim().to_string();
            fields.push((field, value));
        }
    }

    return fields;
}

pub fn safe_rm_file_dir(path: &str) -> Result<(), IOError>{
    if Path::new(&path).is_file() {
        remove_file(&path)?;
    } else {
        match remove_dir(&path){
            Ok(()) => {},
            Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {},
            Err(e) => {
                Err(e)?;
            },
        };
    }

    return Ok(());
}
