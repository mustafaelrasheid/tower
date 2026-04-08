use std::collections::HashSet;
use std::path::Path;
use std::io::Error as IOError;
use crate::utils::{map_atom_to_entries, map_entries_to_atom};
use crate::lock::Lock;
use crate::atom::AtomMetadata;
use crate::group::Group;

fn log_map_atom_to_entries(lock: &Lock, path: &str) {
    for entry in map_atom_to_entries(lock, path, true, &["cache"]) {
        if !Path::new(&entry).exists() {
            println!("Missing: {}", entry);
        }
    }
}

fn log_map_to_lock(path: &str, lock: &Lock, ignore: &Lock)
-> Result<(), IOError> {
    for entry in map_entries_to_atom(path, lock, ignore, false)? {
        println!("Extra: {}", entry);
    }

    return Ok(());
}

pub fn validate_atoms(lock: &Lock, ignore: &Lock, root_dir: &str) 
-> Result<(), IOError> {
    log_map_atom_to_entries(lock, root_dir);
    log_map_to_lock(root_dir, lock, ignore)?;
    
    return Ok(());
}

fn map_to_atoms(groups: &Vec<Group>, atoms: &Vec<AtomMetadata>) {
    let file_atoms: Vec<String> = atoms.iter()
        .map(|a| a.name.to_string())
        .collect();
    
    for group in groups {
        for atom in &group.atoms {
            if !file_atoms.contains(&atom.to_string()) {
                println!(
                    "Group '{}' references non-existent atom '{}'",
                    group.name,
                    atom
                );
            }
        }
    }
}

fn map_to_groups(atoms: &Vec<AtomMetadata>, groups: &Vec<Group>) {
    let grouped_atoms: HashSet<String> = groups
        .iter()
        .map(|g| g.atoms.clone())
        .flatten()
        .collect();
    
    for atom in atoms {
        if !grouped_atoms.contains(&atom.name) {
            println!("Atom '{}' is not in any group", &atom.name);
        }
    }
}

pub fn validate_groups(atoms: &Vec<AtomMetadata>, groups: &Vec<Group>) {
    map_to_atoms(&groups, &atoms);
    map_to_groups(&atoms, &groups);
}
