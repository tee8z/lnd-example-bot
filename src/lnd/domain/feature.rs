use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feature {
    pub name: String,
    pub is_required: bool,
    pub is_known: bool,
}
