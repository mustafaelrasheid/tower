use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Modification {
    #[serde(rename = "replace")]
    Replace,
    #[serde(rename = "exist")]
    Exist,
    #[serde(rename = "cache")]
    Cache,
    #[serde(rename = "slice")]
    Slice,
}

impl AsRef<str> for Modification {
    fn as_ref(&self) -> &str {
        return match self {
            Modification::Replace => "replace",
            Modification::Exist   => "exist",
            Modification::Cache   => "cache",
            Modification::Slice   => "slice",
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    pub contents: HashMap<String, Lock>,
}

impl DirectoryEntry {
    pub fn new() -> Self {
        return Self {
            count:    None,
            contents: HashMap::new()
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub modification: Option<Modification>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}

impl FileEntry {
    pub fn new(
        modification: Option<Modification>,
        file_type: Option<String>,
        count: Option<u32>
    ) -> Self {
        return Self {
            modification: modification,
            file_type:    file_type,
            count:        count
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Lock {
    Dir(DirectoryEntry),
    File(FileEntry),
}

fn walk_and_increment(lock: &mut Lock, atom: &Lock){
    let lock = match lock {
        Lock::Dir(val) => {
            val.count = Some(val.count.unwrap_or(0) + 1);
            val
        },
        Lock::File(val) => {
            val.count = Some(val.count.unwrap_or(0) + 1);
            return;
        }
    };
    
    if let Lock::Dir(atom) = atom {
        for (key, value) in &atom.contents {
            if !lock.contents.contains_key(key) {
                lock.contents.insert(
                    key.clone(),
                    value.clone()
                );
            }
            walk_and_increment(
                &mut lock.contents
                    .get_mut(key)
                    .unwrap(),
                &value
            );
        }
    }
}

pub fn build_lock(atoms: &Vec<Lock>) -> Lock {
    let mut lock = Lock::Dir(DirectoryEntry::new());

    for atom in atoms {
        walk_and_increment(
            &mut lock,
            &atom
        );
    }

    return lock;
}
