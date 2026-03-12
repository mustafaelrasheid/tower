use std::fs::{create_dir_all, read_to_string, write};
use std::path::Path;
use std::io::Read;
use crate::utils::{parse_control, decompress_package};
use crate::error::NetworkError;

const PACKAGES_INDEX_URL: &str =
    "https://deb.debian.org/debian/dists/stable/main/binary-amd64/Packages.gz";
const CACHE_DIR: &str = "/tmp/tower";
const PACKAGES_FILE: &str = "/tmp/tower/Packages";

fn fetch_packages() -> Result<Vec<u8>, NetworkError>{
    let mut compressed = Vec::new();

    ureq::get(PACKAGES_INDEX_URL)
        .call()?
        .into_body()
        .into_reader()
        .read_to_end(&mut compressed)?;

    return Ok(compressed);
}

fn get_packages() -> Result<String, NetworkError> {
    if Path::new(PACKAGES_FILE).exists() {
        if let Ok(packages) = read_to_string(PACKAGES_FILE) {
            return Ok(packages);
        }
    }

    let packages = decompress_package(
        &fetch_packages()?
    ).expect("Failed to decompress Packages");
    
    if let Ok(_) = create_dir_all(CACHE_DIR) {
        let _ = write(
            PACKAGES_FILE,
            &packages.clone()
        );
    };

    return Ok(
        String::from_utf8(packages)
            .expect("Malformed utf8")
            .to_string()
    );
}

fn find_package_filename(package_name: &str)
-> Result<Option<String>, NetworkError> {
    let content = get_packages()?;
    
    for entry in content.split("\n\n") {
        let fields = parse_control(entry);
        let name = if let Some((_, name)) =
            fields.iter().find(|(key, _)| key == "Package") {
                name
            } else { continue; };

        if name != package_name {
            continue;
        }
        if let Some((_, filename)) = 
            fields.iter().find(|(key, _)| key == "Filename") {
            return Ok(Some(filename.clone()));
        }
        return Ok(None);
    }
    
    return Ok(None);
}

pub fn get_deb(package: &str)
-> Result<Option<Vec<u8>>, NetworkError> {
    let mut deb_data = Vec::new();
    let filename = if let Some(name) = find_package_filename(package)? {
        name
    } else {
        return Ok(None);
    };
    let url = format!("https://deb.debian.org/debian/{}", filename);
    
    ureq::get(&url)
        .call()?
        .into_body()
        .into_reader()
        .read_to_end(&mut deb_data)?;
    
    return Ok(Some(deb_data));
}
