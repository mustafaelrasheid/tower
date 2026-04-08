use crate::utils::{map_atom_to_entries, uncover_archive, find_entry}; 
use crate::atom::AtomMetadata;
use crate::error::InvalidInput;

pub fn install_brick(
    lib_dir: &str,
    root_dir: &str,
    package: &[u8],
) -> Result<
    (Vec<(String, u32, Vec<u8>)>, Vec<(String, u32, Vec<u8>)>),
    InvalidInput
> {
    let files = uncover_archive(package)?;
    let mut replace_files: Vec<(String, u32, Vec<u8>)> = Vec::new();
    let mut exist_files:   Vec<(String, u32, Vec<u8>)> = Vec::new();
    let metadata_buf = find_entry(&files, &["metadata.json"])?;
    let metadata: AtomMetadata = serde_json::from_value(
        serde_json::from_str(
            &String::from_utf8_lossy(&metadata_buf).to_string()
        )?
    ).unwrap();
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

    replace_files.push((
        format!("{}/atoms/{}.json", lib_dir, &metadata.name),
        0o644,
        metadata_buf.to_vec()
    ));

    for (path, perm, data) in files {
        let path = path
            .trim_start_matches("contents")
            .to_string();

        if path.as_str() == "metadata.json" {
            continue;
        }
        if replace_entries.contains(&path) {
            replace_files.push((
                format!("{}/{}", root_dir, &path),
                perm,
                data
            ));
            continue;
        }
        if exist_entries.contains(&path) {
            exist_files.push((
                format!("{}/{}", root_dir, &path),
                perm,
                data
            ));
        }
    }

    return Ok((replace_files, exist_files));
}
