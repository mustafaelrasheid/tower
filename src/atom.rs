use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::lock::{Lock, DirectoryEntry};

#[derive(Clone)]
pub struct Atom {
    pub metadata: AtomMetadata,
    pub files: Vec<(String, u32, Vec<u8>)>,
}

impl Atom {
    pub fn new(
        metadata: AtomMetadata,
        files: Vec<(String, u32, Vec<u8>)>
    ) -> Self {
        return Self {
            metadata: metadata,
            files: files
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AtomMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends: Option<Vec<String>>,
    pub contents: HashMap<String, Lock>,
}

impl AtomMetadata {
    pub fn new(
        name: &str,
        description: Option<String>,
        version: Option<String>,
        architecture: Option<String>,
        maintainer: Option<String>,
        section: Option<String>,
        priority: Option<String>,
        homepage: Option<String>,
        depends: Option<Vec<String>>,
        contents: Option<HashMap<String, Lock>>
    ) -> Self {
        return Self {
            name: String::from(name),
            description: description,
            version: version,
            architecture: architecture,
            maintainer: maintainer,
            section: section,
            priority: priority,
            homepage: homepage,
            depends: depends,
            contents: contents.unwrap_or(HashMap::new())
        }
    }
}

impl From<AtomMetadata> for Lock {
    fn from(metadata: AtomMetadata) -> Self {
        return Lock::Dir(
            DirectoryEntry {
                count: None,
                contents: metadata.contents
            }
        );
    }
}
