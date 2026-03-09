use std::io::Error as IOError;
use serde_json::Value;
use crate::utils::{map_atom_to_entries, map_entries_to_atom};
use crate::atom::AtomMetadata;
use crate::lock::Lock;

pub fn purge_atom(
    lib_dir: &str,
    root_dir: &str,
    ignore: &Lock,
    metadata: &AtomMetadata,
    lock: &Lock
) -> Result<Vec<String>, IOError> {
    let mut output: Vec<String> = Vec::new();
    let extra_entries = map_entries_to_atom(
        root_dir,
        &lock,
        &ignore,
        true
    )?;
    let atom_entries = map_atom_to_entries(
        &metadata.clone().into(),
        root_dir,
        true,
        &[]
    );

    for atom_entry in atom_entries.clone().iter().rev() {
        if extra_entries.contains(&atom_entry) {
            output.push(atom_entry.to_string());
        }
    }
    output.push(
        format!(
            "{}/atoms/{}.json",
            lib_dir,
            metadata.name
        )
    );

    return Ok(output);
}
