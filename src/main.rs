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
use std::fs::{read, write, remove_file};
use std::path::Path;
use dialoguer::Confirm;
use clap::Parser;
use rayon::prelude::*;
use crate::args::{Cli, Commands};
use crate::atom::AtomMetadata;
use crate::utils::{
    read_collection_as_json,
    read_file_as_json,
    safe_rm_file_dir,
    safe_place_entry,
    create_package
};
use crate::lock::Lock;
use crate::group::Group;
use crate::atom::Atom;
use crate::error::{InputError, MissingInput};

trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> T;
}

impl<T, E: std::fmt::Display> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> T {
        return match self {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };
    }
}

trait ExpectOrExit<T> {
    fn expect_or_exit(self, msg: &str) -> T;
}

impl<T> ExpectOrExit<T> for Option<T> {
    fn expect_or_exit(self, msg: &str) -> T {
        return match self {
            Some(val) => val,
            None => {
                eprintln!("{}", msg);
                exit(1);
            }
        };
    }
}

impl<T, E: std::fmt::Display> ExpectOrExit<T> for Result<T, E> {
    fn expect_or_exit(self, msg: &str) -> T {
        return match self {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}: {}", msg, e);
                exit(1);
            }
        };
    }
}


fn rebuild_lock(lib_dir: &str, atoms: &Vec<AtomMetadata>) {
    write(
        &format!("{}/lock.json", lib_dir),
        &serde_json::to_string_pretty(
            &lock::build_lock(
                &atoms
                .iter()
                .map(|a| a.clone().into())
                .collect::<Vec<Lock>>()
            )
        ).unwrap()
    ).expect_or_exit("Failed to rebuild lock");
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

    exit(1);
}

fn get_atoms(lib_dir: &str) -> Vec<AtomMetadata> {
    let atoms: Vec<AtomMetadata> = read_collection_as_json(
            &format!("{}/atoms", lib_dir)
        )
        .expect_or_exit("Failed to read atoms directory")
        .into_iter()
        .map(|atom| {
            let val: AtomMetadata = serde_json::from_value(atom)
                .expect_or_exit("Invalid Json format for atom");
            return val;
        })
        .collect();
    
    return atoms;
}

fn get_ignore(lib_dir: &str) -> Lock {
    let ignore: Lock = 
        serde_json::from_value(
            read_file_as_json(&format!("{}/ignore.json", lib_dir))
                .expect_or_exit("Failed to read ignore.json")
        )
        .expect_or_exit("Invalid Json format for ignore.json");

    return ignore;
}

fn get_groups(lib_dir: &str) -> Vec<Group> {
    let groups = read_collection_as_json(&format!("{}/groups", lib_dir))
        .expect_or_exit("Failed to read groups directory")
        .into_iter()
        .map(|group| {
            let val: Group = serde_json::from_value(group)
                .expect_or_exit("Invalid Json format for group");
            return val;
        })
        .collect();

    return groups;
}

fn get_lock(lib_dir: &str) -> Lock {
    let lock: Lock = 
        serde_json::from_value(
            read_file_as_json(&format!("{}/lock.json", lib_dir))
                .expect_or_exit("Failed to read lock.json")
        ).expect_or_exit("Invalid Json format for lock.json");

    return lock;
}

fn output(file_name: &str, file: &[u8]) {
    write(file_name, file)
        .expect_or_exit(&format!("Cannot write file {}", file_name));
}

fn get_group(lib_dir: &str, group: &str) -> Group {
    let group: Group =
        serde_json::from_value(
            read_file_as_json(&format!("{}/groups/{}.json", lib_dir, group))
                .expect_or_exit("Failed to read group")
        )
        .expect_or_exit("Invalid Json format for group specified");

    return group;
}

fn put_group(lib_dir: &str, group: &Group) {
    write(
        &format!("{}/groups/{}.json", lib_dir, group.name),
        &serde_json::to_string_pretty(group).unwrap()
    ).expect_or_exit("Failed to write group");
}

fn main() -> Result<(), InputError> {
    let cli = Cli::parse();
        
    match cli.command {
        Commands::RebuildLock { lib_dir } => {
            let atoms = get_atoms(&lib_dir);

            rebuild_lock(
                &lib_dir,
                &atoms
            );
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
            ).map_err(
                MissingInput::from
            ).map_err(
                InputError::from
            ).unwrap_or_exit();
        },
        Commands::Export { packages, lib_dir, root_dir } => {
            let atoms = get_atoms(&lib_dir);

            for package in packages {
                let atom = atoms
                    .iter()
                    .find(|val| val.name.as_str() == package)
                    .expect_or_exit(
                        &format!("No package exists with name {}", package)
                    );

                output(
                    &format!("{}.brick", package),
                    &export::export(
                        &root_dir,
                        &atom
                    ).map_err(
                        MissingInput::from
                    ).map_err(
                        InputError::from
                    ).unwrap_or_exit()
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
                    &read(&package)
                        .map_err(MissingInput::from)
                        .map_err(InputError::from)
                        .unwrap_or_exit(),
                )?;

                confirm_pkgs_action(
                    yes,
                    "The following files are going to be added or replaced",
                    &replace_entries
                        .iter()
                        .map(|entry| entry.path.to_string())
                        .collect::<Vec<String>>()
                );
                for entry in replace_entries {
                    safe_place_entry(&entry)
                        .expect_or_exit(
                            &format!("Failed to place file {}", &entry.path)
                        );
                }

                confirm_pkgs_action(
                    yes,
                    "The following files are going to be added if not found",
                    &exist_entries
                        .iter()
                        .map(|entry| entry.path.to_string())
                        .collect::<Vec<String>>()
                );
                for entry in exist_entries {
                    if !Path::new(&entry.path).exists() || force {
                        safe_place_entry(&entry)
                            .expect_or_exit(
                                &format!(
                                    "Failed to place file {}",
                                    &entry.path)
                            );
                    }
                }
            }
            rebuild_lock(
                &lib_dir,
                &atoms
            );
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
                    .find(|a| a.name.as_str() == package)
                    .expect_or_exit(
                        &format!("No Package exists with name {}", package)
                    );
                let entries = purge::purge_atom(
                    &lib_dir,
                    &root_dir,
                    &ignore,
                    atom,
                    &lock::build_lock(
                        &atoms
                            .iter()
                            .filter(|a| a.name.as_str() != package)
                            .map(|a| a.clone().into())
                            .collect::<Vec<Lock>>()
                    )
                ).map_err(
                    MissingInput::from
                ).map_err(
                    InputError::from
                ).unwrap_or_exit();

                confirm_pkgs_action(
                    yes,
                    "The following files are going to be deleted",
                    &entries
                );
                
                for entry in entries {
                    safe_rm_file_dir(&entry)
                        .expect_or_exit(
                            &format!("Failed to remove {}", &entry)
                        )
                }
            }
            rebuild_lock(
                &lib_dir,
                &get_atoms(&lib_dir)
            );
        },
        Commands::Convert { packages, deps } => {
            let mut deps = deps
                .iter()
                .map(|dep|
                    read(&dep)
                        .map_err(MissingInput::from)
                        .map_err(InputError::from)
                )
                .collect::<Result<Vec<Vec<u8>>, InputError>>()
                .unwrap_or_exit()
                .par_iter()
                .map(|dep|
                    convert::extract_deb(&dep)
                        .map_err(InputError::from)
                )
                .collect::<Result<Vec<Atom>, InputError>>()
                .unwrap_or_exit();

            for package in packages {
                deps.push(
                    convert::extract_deb(
                        &read(&package)
                            .map_err(MissingInput::from)
                            .map_err(InputError::from)
                            .unwrap_or_exit()
                    ).unwrap_or_exit()
                );

                let (pkg, missing) = convert::convert_deb(
                    &read(&package)
                        .map_err(MissingInput::from)
                        .map_err(InputError::from)
                        .unwrap_or_exit(),
                    &deps
                ).expect_or_exit(&format!("Failed to export {}", &package));
                for miss in missing {
                    println!("package {} was not found", miss);
                }
                output(
                    &format!("{}.brick", package),
                    &create_package(&pkg)
                );
            }
        },
        Commands::GetDeb { packages } => {
            for package in packages {
                let mut unresolved_deps: Vec<String> = Vec::new();
                let mut deps: Vec<Atom> = Vec::new();
                let main_pkg = fetch::get_deb(&package)
                    .expect_or_exit(
                        &format!("Failed to get {} from deb", package)
                    )
                    .expect_or_exit(
                        &format!("Package {} doesn't exist", package)
                    );

                println!("Downloaded package {}", package);

                let pkg = loop {
                    let (pkg, missing) = convert::convert_deb(&main_pkg, &deps)
                        .expect_or_exit(&format!("Failed to export {}", &package));
                    let mut unextracted_deps: Vec<Vec<u8>> = Vec::new();

                    if missing.is_empty()
                    || missing.iter().all(|m| unresolved_deps.contains(m)) {
                        break pkg;
                    }

                    for miss in missing {
                        match &fetch::get_deb(&miss)
                            .expect_or_exit(
                                &format!("Failed to get {} from deb", miss)) {
                            Some(val) => {
                                println!("Downloaded package {}", miss);
                                unextracted_deps.push(
                                    val.to_vec()
                                );
                            },
                            None => {
                                println!(
                                    "Package {} not found, continuing",
                                    miss);
                                unresolved_deps.push(miss);
                            }
                        }
                    }
                    deps.extend(
                        unextracted_deps
                            .par_iter()
                            .map(|dep|
                                convert::extract_deb(dep)
                                    .unwrap_or_exit()
                            )
                            .collect::<Vec<Atom>>()
                    )
                };
                output(
                    &format!("{}.brick", package),
                    &create_package(&pkg)
                );
            }
        },
        Commands::Tag { package, group, lib_dir } => {
            let mut group = get_group(&lib_dir, &group);
            
            group.atoms.insert(package.to_string());
            put_group(&lib_dir, &group);
        },
        Commands::Untag { package, group, lib_dir } => {
            let mut group = get_group(&lib_dir, &group);
            
            group.atoms.remove(&package);
            put_group(&lib_dir, &group);
        },
        Commands::CreateGroup { group, lib_dir } => {
            let group = Group::new(&group);

            put_group(&lib_dir, &group);
        },
        Commands::ListGroup { group, lib_dir } => {
            let group = get_group(&lib_dir, &group);
            
            for item in group.atoms {
                println!("{}", item);
            }
        },
        Commands::DeleteGroup { group, lib_dir } => {
            remove_file(
                &format!("{}/groups/{}.json", lib_dir, group)
            ).unwrap_or_exit();
        }
    }

    return Ok(());
}
