use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::lock::{Lock, DirectoryEntry};
use crate::error::{InvalidInput};

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
pub enum TriggerType {
    #[serde(rename = "interest")]
    Interest,
    #[serde(rename = "interest-await")]
    InterestAwait,
    #[serde(rename = "interest-noawait")]
    InterestNoawait,
    #[serde(rename = "activate")]
    Activate,
    #[serde(rename = "activate-await")]
    ActivateAwait,
    #[serde(rename = "activate-noawait")]
    ActivateNoawait
}

impl TryFrom<&str> for TriggerType {
    type Error = InvalidInput;
    
    fn try_from(text: &str) -> Result<Self, Self::Error> {
        return match text {
            "interest"         => Ok(TriggerType::Interest),
            "interest-await"   => Ok(TriggerType::InterestAwait),
            "interest-noawait" => Ok(TriggerType::InterestNoawait),
            "activate"         => Ok(TriggerType::Activate),
            "activate-await"   => Ok(TriggerType::ActivateAwait),
            "activate-noawait" => Ok(TriggerType::ActivateNoawait),
            _ => Err(
                InvalidInput::FormatSupport(
                    "unknown interest type".to_string()
                )
            )
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub kind: TriggerType,
    pub name: String,
}

impl Trigger {
    pub fn new(
        name: &str,
        kind: TriggerType,
    ) ->Self {
        return Self {
            name: name.to_string(),
            kind: kind,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Shlib {
    name: String,
    major: u32,
    pkg_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl Shlib {
    pub fn new(
        name: &str,
        major: u32,
        pkg_name: &str,
        version: Option<String>
    ) -> Self {
        return Self {
            name: name.to_string(),
            major: major,
            pkg_name: pkg_name.to_string(),
            version: version,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggers: Option<Vec<Trigger>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shlibs: Option<Vec<Shlib>>,
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
        contents: Option<HashMap<String, Lock>>,
        triggers: Option<Vec<Trigger>>,
        shlibs: Option<Vec<Shlib>>,
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
            contents: contents.unwrap_or(HashMap::new()),
            triggers: triggers,
            shlibs: shlibs
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
