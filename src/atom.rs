use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::lock::{Lock, DirectoryEntry};

#[derive(Clone)]
pub struct Atom {
    pub metadata: AtomMetadata,
    pub files: Vec<(String, u32, Vec<u8>)>,
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct AtomMetadata {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends: Option<Vec<String>>,
    pub contents: HashMap<String, Lock>,
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
