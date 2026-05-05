use std::string::FromUtf8Error;
use std::collections::{HashMap, HashSet};
use crate::utils::{
    uncover_archive,
    parse_control,
    find_entry_as_regular
};
use crate::error::{ArchiveError, InvalidInput};
use crate::atom::{Atom, AtomMetadata, Entry, EntryType};
use crate::lock::{Lock, Modification, FileEntry, DirectoryEntry};

fn map_control_to_atom(
    control: &Vec<(String, String)>,
    conffiles: &Vec<String>,
    copyright: Option<String>,
    md5sums: HashMap<String, String>,
    entries: &Vec<Entry>
) -> AtomMetadata {
    let mut metadata = AtomMetadata::new(
        "", None, None, None, None, None, None, None, None, copyright, None);
    
    for (field, value) in control {
        match field.as_str() {
            "Package" => {
                metadata.name = value
                    .as_str()
                    .to_string();
            },
            "Description" => {
                metadata.description = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Version" => {
                metadata.version = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Architecture" => {
                metadata.architecture = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Maintainer" => {
                metadata.maintainer = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Section" => {
                metadata.section = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Priority" => {
                metadata.priority = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Homepage" => {
                metadata.homepage = Some(
                    value
                        .as_str()
                        .to_string()
                );
            },
            "Depends" => {
                metadata.depends = Some(
                    value
                        .split(',')
                        .map(|dep| { dep
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("")
                            .to_string()
                        })
                        .filter(|dep| !dep.is_empty())
                        .collect()
                );
            },
            _ => {}
        }
    }
    
    for entry in entries {
        let modification = if conffiles.contains(&entry.path) {
            Modification::Exist
        } else {
            Modification::Replace
        };
        
        if let Some(md5sum) = md5sums.get(&entry.path)
        && let EntryType::Regular(data) = &entry.data {
            if &format!("{:x}", md5::compute(&data)) != md5sum {
                println!("Warning: Failed md5 checksum for file {}", entry.path);
            }
        }

        insert_path(
            &mut metadata.contents,
            &entry.path,
            modification,
            md5sums.get(&entry.path).cloned()
        );
    }
    
    return metadata;
}

fn insert_path(
    contents: &mut HashMap<String, Lock>,
    path: &str,
    modification: Modification,
    md5sum: Option<String>
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
            Lock::File(
                FileEntry::new(
                    Some(modification),
                    None,
                    None,
                    md5sum
                )
            )
        );
        return;
    }
    
    if !contents.contains_key(part) {
        contents.insert(
            part.clone(),
            Lock::Dir(DirectoryEntry::new())
        );
    }
    
    if let Lock::Dir(dir) = contents.get_mut(part).unwrap() {
        insert_path(
            &mut dir.contents,
            &parts[1..].join("/"),
            modification,
            md5sum
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
-> Result<(
    Vec<(String, String)>,
    Vec<String>,
    Option<String>,
    HashMap<String, String>
), InvalidInput> {
    let archive = uncover_archive(content)?;
    let control = find_entry_as_regular(&archive, &["control"])?;
    let conffiles = match find_entry_as_regular(&archive, &["conffiles"]) {
        Ok(data) => {
            String::from_utf8(data.to_vec())?
                .lines()
                .map(|line| line.trim().trim_start_matches("/").to_string())
                .filter(|line| !line.is_empty())
                .collect()
        },
        Err(_) => {
            Vec::new()
        }
    };
    let md5sums: HashMap<String, String> = match find_entry_as_regular(&archive, &["md5sums"]) {
        Ok(data) => {
            String::from_utf8(data.to_vec())?
                .lines()
                .map(|line| line.split("  ").map(|s| s.to_string()).collect::<Vec<String>>())
                .map(|line| (line[1].trim().trim_start_matches("/").to_string(), line[0].clone()))
                .filter(|line| !line.0.is_empty() || !line.1.is_empty())
                .collect()
        },
        Err(_) => {
            HashMap::new()
        }
    };

    let copyright = match find_entry_as_regular(&archive, &["copyright"]) {
        Ok(text) => {
            Some(String::from_utf8(text.to_vec())?)
        },
        Err(_) => {
            None
        }
    };
    
    return Ok((
        parse_control(&String::from_utf8(control.to_vec())?),
        conffiles,
        copyright,
        md5sums
    ));
}

fn dpkg_data(content: &[u8])
-> Result<Vec<Entry>, ArchiveError> {
    return uncover_archive(&content);
}

pub fn extract_deb(package: &[u8])
-> Result<Atom, InvalidInput> {
    let entries = uncover_archive(package)?;
    let version = dpkg_version(
        find_entry_as_regular(
            &entries,
            &["debian-binary"]
        )?
    )?;
    if &version != "2.0" {
        return Err(
            InvalidInput::FormatSupport(
                "Unknown debian package version"
            .to_string())
        );
    }
    let (control, conffiles, copyright, md5sums) = dpkg_control(
        find_entry_as_regular(
            &entries,
            &["control.tar.gz", "control.tar.xz"]
        )?
    )?;
    let data = dpkg_data(
        find_entry_as_regular(
            &entries,
            &["data.tar.gz", "data.tar.xz"]
        )?
    )?;
    
    return Ok(
        Atom::new(
            map_control_to_atom(
                &control,
                &conffiles,
                copyright,
                md5sums,
                &data,
            ),
            data
        )
    );
}

fn resolve_recursive(
    package: &AtomMetadata,
    available: &HashMap<String, AtomMetadata>,
) -> Result<(HashSet<String>, HashSet<String>), InvalidInput> {
    let mut missing = HashSet::new();
    let mut processed = HashSet::new();
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
                missing.insert(dep);
            }
        }
    }
    
    return Ok((processed, missing));
}

pub fn resolve_deps(
    package: &mut Atom,
    deps: &[Atom]
) -> Result<HashSet<String>, InvalidInput> {
    let deps_metadata: HashMap<String, AtomMetadata> = deps
        .iter()
        .map(|atom| (
            atom.metadata.name.to_string(),
            atom.metadata.clone()
        ))
        .collect();
    let (processed, missing) = resolve_recursive(
        &package.metadata,
        &deps_metadata,
    )?;

    for processed_pkg_name in processed {
        let added_dep = deps.iter()
            .find(|dep_atom| &processed_pkg_name == &dep_atom.metadata.name);
        let added_dep = if let Some(val) = added_dep {
            val
        } else {
            continue;
        };
        
        package.entries.append(&mut added_dep.entries.clone());
        added_dep.entries
            .iter()
            .for_each(|entry| insert_path(
                &mut package.metadata.contents,
                &entry.path,
                Modification::Exist,
                None
            ));
    }
    
    return Ok(missing);
}

pub fn convert_deb(package: &[u8], deps: &[Atom])
-> Result<(Atom, HashSet<String>), InvalidInput> {
    let mut package = extract_deb(package)?;
    let missing = resolve_deps(&mut package, &deps)?;
    
    return Ok((package, missing));
}

