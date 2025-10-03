use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod assign_number;
pub mod favs;
pub mod favs_mode;
pub mod filter;
pub mod help;
pub mod navigate;

#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct FavSessionInfo {
    pub name: String,
    pub is_active: bool,
    pub assigned_number: Option<u8>,
    pub tabs: usize,
    pub panes: usize,
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug)]
pub enum FavsCommandType {
    ReadCache,
    WriteCache,
}

impl Display for FavsCommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FavsCommandType::ReadCache => write!(f, "ReadCache"),
            FavsCommandType::WriteCache => write!(f, "WriteCache"),
        }
    }
}

impl From<&String> for FavsCommandType {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "ReadCache" => FavsCommandType::ReadCache,
            "WriteCache" => FavsCommandType::WriteCache,
            _ => FavsCommandType::ReadCache,
        }
    }
}

impl FavsCommandType {
    pub fn get_command_key() -> String {
        "command_type".to_string()
    }
}
