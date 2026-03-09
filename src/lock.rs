use std::error::Error;
use std::collections::HashMap;
use serde_json::{json, Value};
use serde::{Serialize, Deserialize};

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
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
        match self {
            Modification::Replace => "replace",
            Modification::Exist => "exist",
            Modification::Cache => "cache",
            Modification::Slice => "slice",
        }
    }
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct DirectoryEntry{
    pub count: Option<u32>,
    pub contents: HashMap<String, Lock>,
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub modification: Option<Modification>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub count: Option<u32>
}

#[derive(Serialize, Deserialize)]
#[derive(Clone)]
#[serde(untagged)]
pub enum Lock {
    Dir(DirectoryEntry),
    File(FileEntry)
}

fn walk_and_increment(lock: &mut Lock, atom: &Lock){
    let mut lock = match lock {
        Lock::Dir(val) => {
            val
        },
        Lock::File(val) => {
            val.count = if let Some(count) = val.count {
                Some(count + 1)
            } else { Some(1) };
            return;
        }
    };

    if lock.count == None {
        lock.count = Some(0);
    }

    lock.count = Some(lock.count.unwrap() + 1);
    
    match atom {
        Lock::Dir(contents) => {
            for (key, value) in &contents.contents {
                if !lock.contents.contains_key(key) {
                    lock.contents.insert(
                        key.clone(),
                        value.clone()
                    );
                }
                walk_and_increment(
                    &mut lock.contents.get_mut(key).unwrap(),
                    &value
                );
            }
        },
        Lock::File(file) => {
        }
    }
}

pub fn build_lock(atoms: &Vec<Lock>) -> Lock {
    let mut lock = Lock::Dir(
        DirectoryEntry{
            count: None,
            contents: HashMap::new()
        }
    );

    for atom in atoms {
        walk_and_increment(
            &mut lock,
            &atom
        );
    }

    return lock;
}
