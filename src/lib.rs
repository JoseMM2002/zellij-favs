use serde::{Deserialize, Serialize};

pub mod favs;
pub mod favs_mode;
pub mod filter;
pub mod help;
pub mod navigate;

pub const FAVS_TEMPLATE: &str = r#"[]"#;
pub const FAVS_PATH_CACHE: &str = "/cache/favs.json";
pub const FAVS_PATH_TMP: &str = "/tmp/favs.json";
pub const FAVS_PATH_DATA: &str = "/data/favs.json";
pub const FAVS_PATH_HOST: &str = "/host/favs.json";

pub const FAVS_SYNC_MESSAGE_NAME: &str = "favs_sync";

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct FavSessionInfo {
    name: String,
    is_active: bool,
}

pub fn get_fav_path(has_data_dir: bool) -> &'static str {
    if has_data_dir {
        FAVS_PATH_HOST
    } else {
        FAVS_PATH_TMP
    }
}
