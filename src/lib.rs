use serde::{Deserialize, Serialize};

pub mod favs;
pub mod favs_mode;

pub const FAVS_TEMPLATE: &str = r#"[]"#;
pub const FAVS_PATH: &str = "/cache/favs.json";

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct FavSessionInfo {
    name: String,
    is_active: bool,
}
