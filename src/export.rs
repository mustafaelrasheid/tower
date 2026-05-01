use std::path::Path;
use std::fs::{read, read_link};
use std::fs;
use std::io::Error as IOError;
use std::os::unix::fs::PermissionsExt;
use crate::utils::{map_atom_to_entries, create_package};
use crate::atom::{Atom, AtomMetadata, Entry, EntryType};

fn get_files(
    metadata: &AtomMetadata,
    root_dir: &str
) -> Result<Vec<Entry>, IOError> {
    let mut output: Vec<Entry> = Vec::new();
    let entries = map_atom_to_entries(
        &metadata.clone().into(),
        "",
        false,
        &["cache"]
    );

    for entry in entries {
        let path = format!("{}/{}", root_dir, entry);
        let data = if Path::new(&path).is_symlink() {
            EntryType::Symlink(
                read_link(&path)?
                    .to_string_lossy()
                    .to_string()
            )
        } else {
            EntryType::Regular(read(&path)?)
        };

        output.push(
            Entry::new(
                &path,
                fs::metadata(&path)?
                    .permissions()
                    .mode(),
                data
            )
        );
    }

    return Ok(output);
}

pub fn export(root_dir: &str, metadata: &AtomMetadata)
-> Result<Vec<u8>, IOError> {
    let atom = Atom::new(
        metadata.clone(),
        get_files(
            &metadata,
            root_dir
        )?,
    );
    
    return Ok(create_package(&atom));
}
