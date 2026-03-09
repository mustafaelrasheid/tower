use std::error::Error;
use std::collections::HashSet;
use std::path::Path;
use serde_json::{Value};
use crate::utils::{map_atom_to_entries, map_entries_to_atom};
use crate::lock::Lock;
use crate::atom::AtomMetadata;
use crate::group::Group;

fn log_map_atom_to_entries(lock: &Lock, path: &str)
-> Result<(), Box<dyn Error>> {
    let entries = map_atom_to_entries(lock, path, true, &["cache"]);

    for entry in entries {
        if !Path::new(&entry).exists() {
            println!("Missing: {}", entry);
        }
    }

    return Ok(());
}

fn log_map_to_lock(path: &str, lock: &Lock, ignore: &Lock)
-> Result<(), Box<dyn Error>> {
    let entries = map_entries_to_atom(path, lock, ignore, false)?;


    for entry in entries {
        println!("Extra: {}", entry);
    }

    return Ok(());
}

pub fn validate_atoms(lock: &Lock, ignore: &Lock, root_dir: &str) 
-> Result<(), Box<dyn Error>> {
    log_map_atom_to_entries(lock, root_dir)
        .unwrap_or_else(|e| {
            eprintln!("Failed to map lock to files due to {}", e);
        });
    log_map_to_lock(root_dir, lock, ignore)
        .unwrap_or_else(|e| {
            eprintln!("Failed to map files to lock due to {}", e);
        });
    
    return Ok(());
}

fn map_to_atoms(groups: &Vec<Group>, atoms: &Vec<AtomMetadata>){
    let file_atoms: Vec<String> = atoms.iter()
        .map(|a| 
            a.name.to_string()
        )
        .collect();
    
    for group in groups {
        let group_atoms = &group.atoms;
        
        for atom_ref in group_atoms {
            if !file_atoms.contains(&atom_ref.to_string()) {
                println!(
                    "Group '{}' references non-existent atom '{}'",
                    group.name,
                    atom_ref
                );
            }
        }
    }
}

fn map_to_groups(atoms: &Vec<AtomMetadata>, groups: &Vec<Group>){
    let mut grouped_atoms = HashSet::new();
    
    for group in groups {
        let group_atoms = &group.atoms;

        for atom_ref in group_atoms {
            grouped_atoms.insert(atom_ref.to_string());
        }
    }
    
    for atom in atoms {
        let atom_name = atom.name.clone();

        if !grouped_atoms.contains(&atom_name) {
            println!("Atom '{}' is not in any group", &atom_name);
        }
    }
}

pub fn validate_groups(atoms: &Vec<AtomMetadata>, groups: &Vec<Group>){
    map_to_atoms(&groups, &atoms);
    map_to_groups(&atoms, &groups);
}
