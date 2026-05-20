use std::collections::BTreeMap;

use zellij_tile::prelude::run_command;

use super::{Favs, FavsJson};
use crate::FavsCommandType;

impl Favs {
    pub fn commit_fav_changes(&self) {
        let favs_info: FavsJson = self.into();
        let json = serde_json::to_string(&favs_info).unwrap();
        let mut data = BTreeMap::new();
        data.insert(
            FavsCommandType::get_command_key(),
            FavsCommandType::WriteCache.to_string(),
        );

        run_command(
            &[
                "bash",
                "-c",
                format!("echo '{}' > {}", json, self.cache_dir).as_str(),
            ],
            data,
        );
    }

    pub fn load_cache(&self) {
        let mut data = BTreeMap::new();
        data.insert(
            FavsCommandType::get_command_key(),
            FavsCommandType::ReadCache.to_string(),
        );

        run_command(
            &["bash", "-c", format!("cat {}", self.cache_dir).as_str()],
            data,
        );
    }

    pub fn handle_run_command_result(
        &mut self,
        exit_code: Option<i32>,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        command_type: FavsCommandType,
    ) -> bool {
        if let Some(exit_code) = exit_code
            && exit_code != 0
        {
            let stderr = String::from_utf8(stderr).unwrap_or_default();

            if matches!(command_type, FavsCommandType::ReadCache)
                && stderr.contains("No such file or directory")
            {
                self.error = None;
                self.has_loaded = true;
                self.commit_fav_changes();
                self.refresh_session_list();
                return true;
            }

            self.error = Some(stderr.clone());

            return true;
        }

        if let FavsCommandType::ReadCache = command_type {
            if let Ok(json_string) = String::from_utf8(stdout)
                && let Ok(sessions) = serde_json::from_str::<FavsJson>(&json_string)
            {
                self.fav_sessions = sessions.favs;
                self.flush_sessions = sessions.flush;
            }
            self.has_loaded = true;
            self.error = None;
            self.refresh_session_list();
            return true;
        }

        false
    }
}
