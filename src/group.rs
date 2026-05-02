use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub atoms: HashSet<String>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        return Self {
            name: name.to_string(),
            atoms: HashSet::new(),
        }
    }
}
