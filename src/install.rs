use crate::utils::{
    map_atom_to_entries,
    uncover_archive,
    find_entry_as_regular
}; 
use crate::atom::{AtomMetadata, Entry, EntryType};
use crate::error::InvalidInput;

pub fn install_brick(
    lib_dir: &str,
    root_dir: &str,
    package: &[u8],
) -> Result<
    (Vec<Entry>, Vec<Entry>),
    InvalidInput
> {
    let entries = uncover_archive(package)?;
    let mut replace_files: Vec<Entry> = Vec::new();
    let mut exist_files:   Vec<Entry> = Vec::new();
    let metadata_buf = find_entry_as_regular(
        &entries,
        &["metadata.json"]
    )?;
    let metadata: AtomMetadata = serde_json::from_value(
        serde_json::from_str(
            &String::from_utf8_lossy(&metadata_buf).to_string()
        )?
    )?;
    let replace_entries = map_atom_to_entries(
        &metadata.clone().into(),
        root_dir,
        false,
        &["cache", "exist"]
    );
    let exist_entries = map_atom_to_entries(
        &metadata.clone().into(),
        root_dir,
        false,
        &["slice", "replace"]
    );

    replace_files.push(
        Entry::new(
            &format!("{}/atoms/{}.json", lib_dir, &metadata.name),
            0o644,
            EntryType::Regular(metadata_buf.to_vec())
        )
    );

    for entry in entries {
        let path = entry.path
            .trim_start_matches("contents")
            .to_string();

        if path.as_str() == "metadata.json" {
            continue;
        }
        if replace_entries.contains(&path) {
            replace_files.push(
                Entry::new(
                    &format!("{}/{}", root_dir, &path),
                    entry.perm,
                    entry.data
                )
            );
            continue;
        }
        if exist_entries.contains(&path) {
            exist_files.push(
                Entry::new(
                    &format!("{}/{}", root_dir, &path),
                    entry.perm,
                    entry.data
                )
            );
        }
    }

    return Ok((replace_files, exist_files));
}
