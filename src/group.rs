use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub atoms: Vec<String>
}
