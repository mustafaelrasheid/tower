mod lock;
mod install;
mod args;
mod validate;
mod utils;
mod export;
mod purge;
mod convert;
mod fetch;
mod error;
mod atom;
mod group;

use std::process::exit;
use std::error::Error;
use std::fs::{
    read,
    remove_dir,
    create_dir_all,
    remove_file,
    write,
    set_permissions,
    Permissions
};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::io::Error as IOError;
use dialoguer::Confirm;
use clap::Parser;
use args::{Cli, Commands};
use serde_json::Value;
use crate::atom::AtomMetadata;
use crate::utils::{
    read_collection_as_json,
    write_file_as_json,
    read_file_as_json,
    safe_rm_file_dir,
    create_package
};
use crate::lock::Lock;
use crate::group::Group;
use crate::atom::Atom;

fn rebuild_lock(lib_dir: &str, atoms: &Vec<Lock>) 
-> Result<(), IOError> {
    write(
        &format!("{}/lock.json", lib_dir),
        &serde_json::to_string_pretty(
            &lock::build_lock(atoms)
        ).unwrap()
    )?;

    return Ok(());
}

fn confirm_pkgs_action(yes: bool, prompt: &str, packages: &Vec<String>) {
    if yes {
        return;
    }
    
    println!(
        "{}:{}?",
        prompt,
        &packages
            .into_iter()
            .map(|package| format!("\n\t\"{}\"", package))
            .collect::<Vec<String>>()
            .join(", ")
    );
    
    if Confirm::new()
        .with_prompt("Confirm.")
        .interact()
        .unwrap_or(false) {
        return;
    }

    exit(0);
}

fn get_atoms(lib_dir: &str) -> Vec<AtomMetadata> {
    let atoms: Vec<AtomMetadata> = read_collection_as_json(&format!("{}/atoms", lib_dir))
        .unwrap_or_else(|e| {
            eprintln!("Failed to read atoms directory: {}", e);
            exit(1);
        })
        .into_iter()
        .map(|atom| {
            let val: AtomMetadata = serde_json::from_value(atom)
                .unwrap_or_else(|e| {
                    eprintln!("Invalid Json format for atom: {}", e);
                    exit(1);
                });
            return val;
        })
        .collect();
    
    return atoms;
}

fn get_ignore(lib_dir: &str) -> Lock {
    let ignore: Lock = 
        serde_json::from_value(
            read_file_as_json(&format!("{}/ignore.json", lib_dir))
                .unwrap_or_else(|e| {
                    eprintln!("Failed to read ignore.json: {}", e);
                    exit(1);
                })
        )
        .unwrap_or_else(|e| {
            eprintln!("Invalid Json format for ignore.json: {}", e);
            exit(1);
        });

    return ignore;
}

fn get_groups(lib_dir: &str) -> Vec<Group> {
    let groups = read_collection_as_json(&format!("{}/groups", lib_dir))
        .unwrap_or_else(|e| {
            eprintln!("Failed to read lock.json: {}", e);
            exit(1);
        })
        .into_iter()
        .map(|atom| {
            let val: Group = serde_json::from_value(atom).unwrap_or_else(|e| {
                eprintln!("Invalid Json format for group: {}", e);
                exit(1);
            });
            return val;
        })
        .collect();

    return groups;
}

fn get_lock(lib_dir: &str) -> Lock {
    let lock: Lock = 
        serde_json::from_value(
            read_file_as_json(&format!("{}/lock.json", lib_dir)).unwrap_or_else(|e| {
                eprintln!("Failed to read lock.json: {}", e);
                exit(1);
            })
        ).unwrap_or_else(|e| {
            eprintln!("Invalid Json format for lock.json: {}", e);
            exit(1);
        });

    return lock;
}

fn output(file_name: &str, file: &[u8]) {
    write(file_name, file).unwrap_or_else(|e| {
        eprintln!("Cannot write file {} due to: {}", file_name, e);
        exit(1);
    })
}

fn main() -> Result<(), IOError>{
    let cli = Cli::parse();
        
    match cli.command {
        Commands::RebuildLock { lib_dir } => {
            let atoms = get_atoms(&lib_dir);

            rebuild_lock(
                &lib_dir,
                &atoms.iter().map(|a| a.clone().into()).collect()
            ).unwrap_or_else(|e| {
                eprintln!("Failed to rebuild lock due to {}", e);
                exit(1);
            });
        },
        Commands::Validate { lib_dir, root_dir } => {
            let atoms  = get_atoms(&lib_dir);
            let ignore = get_ignore(&lib_dir);
            let groups = get_groups(&lib_dir);
            let lock   = get_lock(&lib_dir);

            validate::validate_groups(&atoms, &groups);
            validate::validate_atoms(
                &lock,
                &ignore,
                &root_dir
            ).unwrap_or_else(|e| {
                eprintln!("Failed to validate atoms due to: {}", e);
                exit(1);
            });
        },
        Commands::Export { packages, lib_dir, root_dir } => {
            let atoms = get_atoms(&lib_dir);

            for package in packages {
                let atom = atoms
                    .get(
                        atoms.iter()
                            .position(|value| { value.name.as_str() == package })
                            .ok_or("No Atom exists with this name")
                            .unwrap_or_else(|e| {
                                eprintln!("{}", e);
                                exit(1);
                            })
                    )
                    .unwrap();

                output(
                    &format!("{}.brick", package),
                    &export::export(
                        &root_dir,
                        &atom
                    ).unwrap_or_else(|e| {
                        eprintln!("Failed to export package due to {}", e);
                        exit(1);
                    })
                );
            }
        },
        Commands::Install { packages, lib_dir, root_dir, force, yes } => {
            let atoms = get_atoms(&lib_dir);

            confirm_pkgs_action(
                yes,
                "Are you sure you want to install the following package(s)",
                &packages
            );
            
            for package in packages {
                let (replace_entries, exist_entries) = install::install_brick(
                    &lib_dir,
                    &root_dir,
                    &read(&package)?,
                ).unwrap_or_else(|e| {
                    eprintln!("Failed to install package due to {}", e);
                    exit(1);
                });

                confirm_pkgs_action(
                    yes,
                    "The following files are going to be added or replaced",
                    &replace_entries
                        .iter()
                        .map(|(name, perm, _)| name.to_string())
                        .collect::<Vec<String>>()
                );
                confirm_pkgs_action(
                    yes,
                    "The following files are going to be added if not found",
                    &exist_entries
                        .iter()
                        .map(|(name, perm, _)| name.to_string())
                        .collect::<Vec<String>>()
                );

                for (entry, perm, data) in replace_entries {
                    create_dir_all(
                        Path::new(&entry)
                            .parent()
                            .expect("invalid path")
                    )?;
                    write(&entry, &data)?;
                    set_permissions(&entry, Permissions::from_mode(perm))?;
                }
                for (entry, perm, data) in exist_entries {
                    create_dir_all(
                        Path::new(&entry)
                            .parent()
                            .expect("invalid path")
                    )?;
                    if !Path::new(&entry).exists() {
                        write(&entry, &data)?;
                    }
                    set_permissions(&entry, Permissions::from_mode(perm))?;
                }
            }
            rebuild_lock(
                &lib_dir,
                &atoms.iter().map(|a| a.clone().into()).collect()
            ).unwrap_or_else(|e| {
                eprintln!("Failed to rebuild lock due to {}", e);
                exit(1);
            });
        },
        Commands::Purge { packages, lib_dir, root_dir, yes } => {
            let atoms  = get_atoms(&lib_dir);
            let ignore = get_ignore(&lib_dir);

            confirm_pkgs_action(
                yes,
                "Are you sure you want to purge the following package(s)",
                &packages
            );
            
            for package in packages {
                let atom = atoms.iter()
                    .find(|value| { value.name.as_str() == package })
                    .ok_or("No Atom exists with this name")
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to purge package due to: {}", e);
                        exit(1);
                    });
                let mut atoms = atoms.clone();
                
                atoms.remove(
                    atoms.iter()
                        .position(|value| { value.name.as_str() == package })
                        .unwrap()
                );
                let entries = purge::purge_atom(
                    &lib_dir,
                    &root_dir,
                    &ignore,
                    atom,
                    &lock::build_lock(
                        &atoms.iter().map(|a| a.clone().into()).collect()
                    )
                ).unwrap_or_else(|e| {
                    eprintln!("Failed to purge package due to: {}", e);
                    exit(1);
                });

                confirm_pkgs_action(
                    yes,
                    "The following files are going to be deleted",
                    &entries
                );
                
                entries
                    .iter()
                    .for_each(|entry|
                        safe_rm_file_dir(entry)
                            .expect(&format!("Failed to remove {}", entry))
                    );
            }
            let atoms = get_atoms(&lib_dir);
            rebuild_lock(
                &lib_dir,
                &atoms.iter().map(|a| a.clone().into()).collect()
            ).unwrap_or_else(|e| {
                eprintln!("Failed to rebuild lock due to {}", e);
                exit(1);
            });
        },
        Commands::Convert { packages, deps } => {
            let mut deps = deps
                .iter()
                .map(|dep| read(&dep).unwrap())
                .map(|dep| convert::extract_deb(&dep).unwrap())
                .collect::<Vec<Atom>>();

            for package in packages {
                deps.push(convert::extract_deb(&read(&package)?).unwrap());

                let (pkg, missing) = convert::convert_deb(
                    &read(&package)?,
                    &deps
                ).unwrap_or_else(|e| {
                    eprintln!("Failed to export {} due to {}", &package, e);
                    exit(1);
                });
                for miss in missing {
                    println!("package {} was not found", miss);
                }
                output(
                    &format!("{}.brick", package),
                    &create_package(pkg)
                );
            }
        },
        Commands::GetDeb { packages } => {
            for package in packages {
                let mut unresolved_deps: Vec<String> = Vec::new();
                let mut deps: Vec<Atom> = Vec::new();
                let main_pkg = if let Some(pkg) = fetch::get_deb(&package)
                    .unwrap_or_else(|e| {
                        eprintln!(
                            "Failed to get {} from deb due to {}",
                            package, e
                        );
                        exit(1);
                    }) { pkg } else {
                        eprintln!(
                            "Package {} doesn't exist",
                            package
                        );
                        exit(1);
                    };

                println!("Downloaded package {}", package);

                let pkg = loop {
                    let (pkg, missing) = convert::convert_deb(&main_pkg, &deps).unwrap_or_else(|e| {
                        eprintln!("Failed to export {} due to {}", &package, e);
                        exit(1);
                    });
                    
                    if missing.is_empty() || missing.iter().all(|m| unresolved_deps.contains(m)) {
                        break pkg;
                    }
                    for miss in missing {
                        let dep = &fetch::get_deb(&miss)
                            .unwrap_or_else(|e| {
                                eprintln!(
                                    "Failed to get {} from deb due to {}",
                                    miss, e
                                );
                                exit(1);
                            });
                        match dep {
                            Some(dep) => {
                                println!("Downloaded package {}", miss);
                                deps.push(convert::extract_deb(dep).unwrap());
                            },
                            None => {
                                println!("Package {} not found, continueing", miss);
                                unresolved_deps.push(miss);
                            }
                        }

                    }
                };
                output(
                    &format!("./{}.brick", package),
                    &create_package(pkg)
                );
            }
        }
    }

    return Ok(());
}
