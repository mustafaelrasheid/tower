use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::lock::{Lock, DirectoryEntry};

#[derive(Clone)]
pub enum EntryType {
    Symlink(String),
    Regular(Vec<u8>),
}

#[derive(Clone)]
pub struct Entry {
    pub path: String,
    pub perm: u32,
    pub data: EntryType,
}

impl Entry {
    pub fn new(
        path: &str,
        perm: u32,
        data: EntryType
    ) -> Self {
        return Self {
            path: path.to_string(),
            perm: perm,
            data: data
        };
    }
}

#[derive(Clone)]
pub struct Atom {
    pub metadata: AtomMetadata,
    pub entries: Vec<Entry>,
}

impl Atom {
    pub fn new(
        metadata: AtomMetadata,
        entries: Vec<Entry>
    ) -> Self {
        return Self {
            metadata: metadata,
            entries: entries
        };
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
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
        copyright: Option<String>,
        changelog: Option<String>,
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
            copyright: copyright,
            changelog: changelog,
            contents: contents.unwrap_or(HashMap::new())
        };
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
