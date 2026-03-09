use std::string::FromUtf8Error;
use std::collections::{HashMap, HashSet};
use crate::utils::{uncover_archive, create_package, parse_control};
use crate::error::{ArchiveError, InvalidInput};
use crate::atom::{Atom, AtomMetadata};
use crate::lock::{Lock, Modification, FileEntry, DirectoryEntry};

fn map_control_to_atom(
    control: &Vec<(String, String)>,
    files: &Vec<(String, u32, Vec<u8>)>
) -> AtomMetadata {
    let mut name        = String::new();
    let mut description = String::new();
    let mut depends     = Vec::new();
    let mut contents    = HashMap::new();
    
    for (field, value) in control {
        match field.as_str() {
            "Package" => {
                name = value
                    .as_str()
                    .to_string();
            },
            "Description" => {
                description = value
                    .as_str()
                    .to_string();
            },
            "Depends" => {
                depends = value
                    .split(',')
                    .map(|dep| { dep
                        .trim()
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string()
                    })
                    .filter(|dep| !dep.is_empty())
                    .collect();
            },
            _ => {}
        }
    }
    
    for (path, _perm, _data) in files {
        insert_path(
            &mut contents,
            path,
            Modification::Replace
        );
    }
    
    return AtomMetadata{
        name:        name,
        description: Some(description),
        depends:     Some(depends),
        contents:    contents
    };
}

fn insert_path(
    contents: &mut HashMap<String, Lock>,
    path: &str,
    modification: Modification
) {
    let parts: Vec<String> = path
        .trim_start_matches("./")
        .split('/')
        .map(|s| s.to_string())
        .collect();
    
    let part = &parts[0];
    
    if parts.len() == 1 {
        contents.insert(
            part.clone(),
            Lock::File(FileEntry {
                modification: Some(modification),
                file_type: None,
                count: None
            })
        );
        return;
    }
    
    if !contents.contains_key(part) {
        contents.insert(
            part.clone(),
            Lock::Dir(DirectoryEntry {
                contents: HashMap::new(),
                count: None
            })
        );
    }
    
    if let Lock::Dir(dir) = contents.get_mut(part).unwrap() {
        insert_path(
            &mut dir.contents,
            &parts[1..].join("/"),
            modification
        );
    }
}

fn dpkg_version(content: &[u8]) -> Result<String, FromUtf8Error> {
    let content = String::from_utf8(content.to_vec())?
        .trim()
        .to_string();

    return Ok(content);
}

fn dpkg_control(content: &[u8])
-> Result<Vec<(String, String)>, InvalidInput> {
    let archive = uncover_archive(content)?;
    
    let control = archive.iter()
        .find_map(|(name, _perm, data)| {
            if name == "./control" {
                Some(data)
            } else { None }
        })
        .ok_or(InvalidInput::MissingData(
            "Missing control file in control archive"
        .to_string()))?;
    
    return Ok(
        parse_control(&String::from_utf8(control.to_vec())?)
    );
}

fn dpkg_data(content: &[u8])
-> Result<Vec<(String, u32, Vec<u8>)>, ArchiveError> {
    return uncover_archive(&content);
}

pub fn extract_deb(package: &[u8])
-> Result<Atom, InvalidInput> {
    let entries: Vec<(String, u32, Vec<u8>)> = uncover_archive(package)?;
    
    let version = dpkg_version(
        entries.iter().find_map(|(name, _perm, data)| {
            if name == "debian-binary" {
                Some(data)
            } else { None }
        }).ok_or(InvalidInput::MissingData(
            "Missing version file"
        .to_string()))?,
    )?;
    if &version != "2.0" {
        return Err(
            InvalidInput::FormatSupport(
                "Unknown debian package version"
            .to_string())
        );
    }
    let control = dpkg_control(
        entries.iter().find_map(|(name, _perm, data)| {
            if name == "control.tar.gz" || name == "control.tar.xz" {
                Some(data)
            } else { None }
        }).ok_or(InvalidInput::MissingData(
            "Missing control file"
        .to_string()))?,
    )?;
    let data = dpkg_data(
        entries.iter().find_map(|(name, _perm, data)| {
            if name == "data.tar.gz" || name == "data.tar.xz" {
                Some(data)
            } else { None }
        }).ok_or(InvalidInput::MissingData(
            "Missing data file"
        .to_string()))?,
    )?;
    
    return Ok(
        Atom{
            metadata: map_control_to_atom(
                &control,
                &data
            ),
            files: data
        }
    );
}

fn resolve_recursive(
    package: &AtomMetadata,
    available: &HashMap<String, AtomMetadata>,
    add_package: bool
) -> Result<(HashSet<String>, Vec<String>), InvalidInput> {
    let mut missing = Vec::new();
    let mut processed: HashSet<String> = HashSet::new();
    let mut to_process = vec![package.clone()];
    
    while let Some(current) = to_process.pop() {
        if processed.contains(&current.name) {
            continue;
        }
        
        processed.insert(current.name.clone());

        for dep in current.depends.unwrap_or_default() {
            if let Some(dep_meta) = available.get(&dep) {
                if !processed.contains(&dep) {
                    to_process.push(dep_meta.clone());
                }
            } else {
                missing.push(dep);
            }
        }
    }
    
    return Ok((processed, missing));
}

pub fn resolve_deps(
    package: &mut Atom,
    deps: &[Atom]
) -> Result<Vec<String>, InvalidInput> {
    let deps_metadata = deps
        .iter()
        .map(|atom| (
            atom.metadata.name.to_string(),
            atom.metadata.clone()
        ))
        .collect::<HashMap<String, AtomMetadata>>();
    let (processed, missing) = resolve_recursive(
        &package.metadata,
        &deps_metadata,
        false
    )?;

    for processed_pkg_name in processed {
        let added_dep = deps.iter()
            .find(|dep_atom| &processed_pkg_name == &dep_atom.metadata.name);

        let added_dep = if let Some(val) = added_dep {
            val
        } else {
            continue;
        };
        
        package.files.append(&mut added_dep.files.clone());
        added_dep.files
            .iter()
            .for_each(|(entry, _perm, _data)| insert_path(
                &mut package.metadata.contents,
                entry,
                Modification::Exist
            ));
    }
    
    return Ok(missing);
}

pub fn convert_deb(package: &[u8], deps: &[Atom])
-> Result<(Atom, Vec<String>), InvalidInput> {
    let mut package = extract_deb(package)?;
    let missing = resolve_deps(&mut package, &deps)?;
    
    return Ok((package, missing));
}

