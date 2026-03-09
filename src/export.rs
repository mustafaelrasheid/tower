use std::fs::{read};
use std::fs;
use std::io::Error as IOError;
use crate::utils::{map_atom_to_entries, create_package};
use crate::atom::{Atom, AtomMetadata};
use std::os::unix::fs::PermissionsExt;

fn get_files(
    metadata: &AtomMetadata,
    root_dir: &str
) -> Result<Vec<(String, u32, Vec<u8>)>, IOError> {
    let mut output: Vec<(String, u32, Vec<u8>)> = Vec::new();
    let entries = map_atom_to_entries(
        &metadata.clone().into(),
        "",
        false,
        &["cache"]
    );

    for entry in entries {
        let path = format!("{}/{}", root_dir, entry);

        output.push((
            path.clone(),
            fs::metadata(&path)?
                .permissions()
                .mode(),
            read(&path)?
        ));
    }

    return Ok(output);
}

pub fn export(root_dir: &str, metadata: &AtomMetadata)
-> Result<Vec<u8>, IOError> {
    let atom = Atom {
        metadata: metadata.clone(),
        files: get_files(
            &metadata,
            root_dir
        )?,
    };
    
    return Ok(create_package(atom));
}
