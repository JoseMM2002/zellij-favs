use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use zellij_tile::{
    prelude::*,
    shim::{print_text_with_coordinates, request_permission, subscribe, Text},
    ZellijPlugin,
};

use crate::{
    assign_number::match_assignation_keys, favs_mode::FavMode, filter::match_filter_key,
    help::match_help_keys, navigate::match_navigation_keys, FavSessionInfo, FavsCommandType,
};

mod command;
mod render;
mod session;

pub struct Favs {
    pub fav_sessions: Vec<FavSessionInfo>,
    pub flush_sessions: Vec<FavSessionInfo>,
    pub cursor: usize,
    pub mode: FavMode,
    pub current_column: Option<FavMode>,
    pub filter: Option<String>,
    pub has_loaded: bool,
    pub cache_dir: String,
    pub display_tab_panes: bool,
    pub error: Option<String>,
}

impl Default for Favs {
    fn default() -> Self {
        Self {
            has_loaded: false,
            fav_sessions: vec![],
            cursor: 0,
            mode: FavMode::NavigateFavs,
            current_column: None,
            filter: None,
            flush_sessions: vec![],
            cache_dir: String::from("~/.cache/favs.json"),
            display_tab_panes: false,
            error: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavsJson {
    pub favs: Vec<FavSessionInfo>,
    pub flush: Vec<FavSessionInfo>,
}

impl From<&Favs> for FavsJson {
    fn from(val: &Favs) -> Self {
        FavsJson {
            favs: val.fav_sessions.clone(),
            flush: val.flush_sessions.clone(),
        }
    }
}

impl Favs {
    fn match_key(&mut self, key: &BareKey) -> bool {
        match &mut self.mode {
            FavMode::Filter => match_filter_key(self, key),
            FavMode::Help => match_help_keys(self, key),
            FavMode::AssignNumber => match_assignation_keys(self, key),
            _ => match_navigation_keys(self, key),
        }
    }
}

impl ZellijPlugin for Favs {
    fn load(&mut self, configuration: std::collections::BTreeMap<String, String>) {
        if let Some(cache_dir) = configuration.get("cache_dir") {
            self.cache_dir = cache_dir.to_string();
        }
        if let Some(display_tab_panes) = configuration.get("display_tab_panes") {
            self.display_tab_panes = matches!(display_tab_panes.trim(), "true" | "t" | "y" | "1");
        }

        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::PermissionRequestResult,
            EventType::SessionUpdate,
            EventType::RunCommandResult,
        ]);
    }

    fn update(&mut self, event: zellij_tile::prelude::Event) -> bool {
        let mut render = false;
        match event {
            Event::Key(key) => {
                render = self.match_key(&key.bare_key);
            }
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                if !self.has_loaded {
                    self.load_cache();
                } else {
                    render = self.refresh_session_list();
                }
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.error = Some("Required permissions were denied".to_string());
                render = true;
            }
            Event::SessionUpdate(_, _) if self.has_loaded => {
                render = self.refresh_session_list();
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context) => {
                if let Some(command_type) = context
                    .get(FavsCommandType::get_command_key().as_str())
                    .map(FavsCommandType::from)
                {
                    render =
                        self.handle_run_command_result(exit_code, stdout, stderr, command_type);
                }
            }
            _ => {}
        }

        render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        match self.mode {
            FavMode::Help => {
                self.render_help_commands();
            }
            _ => self.render_navigation(cols, rows),
        }
        if let Some(error) = &self.error {
            let error_label = format!("Error: {}", error);
            let error_text = format!("{}", error_label.red().bold());
            let x = cols.saturating_sub(error_label.chars().count());
            let y = rows.saturating_sub(1);
            print_text_with_coordinates(Text::new(error_text), x, y, None, None);
        }
    }
}
