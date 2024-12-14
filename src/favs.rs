use std::{fs::File, io::Write};

use owo_colors::OwoColorize;
use zellij_tile::{
    prelude::*,
    shim::{print_text_with_coordinates, request_permission, subscribe, Text},
    ZellijPlugin,
};

use crate::{
    favs_mode::FavMode, filter::match_filter_key, help::match_help_keys,
    navigate::match_navigation_keys, FavSessionInfo, FAVS_PATH_TMP, FAVS_TEMPLATE,
};

pub struct Favs {
    pub fav_sessions: Vec<FavSessionInfo>,
    pub flush_sessions: Vec<FavSessionInfo>,
    pub cursor: usize,
    pub mode: FavMode,
    pub filter: Option<String>,
}

impl Default for Favs {
    fn default() -> Self {
        if !std::path::Path::new(FAVS_PATH_TMP).exists() {
            let create = File::create(FAVS_PATH_TMP);
            let mut file = create.unwrap();
            file.write_all(FAVS_TEMPLATE.as_bytes()).unwrap();
        }

        Self {
            fav_sessions: vec![],
            cursor: 0,
            mode: FavMode::NavigateFavs,
            filter: None,
            flush_sessions: vec![],
        }
    }
}

impl Favs {
    fn match_key(&mut self, key: &BareKey) -> bool {
        match &mut self.mode {
            FavMode::Filter => match_filter_key(self, key),
            FavMode::Help => match_help_keys(self, key),
            _ => match_navigation_keys(self, key),
        }
    }
    pub fn commit_fav_changes(&self) {
        let favs_to_save: Vec<String> = self
            .fav_sessions
            .iter()
            .map(|session| session.name.clone())
            .collect();
        let json = serde_json::to_string(&favs_to_save).unwrap();

        let mut file = File::create(FAVS_PATH_TMP).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
    pub fn get_filtered_sessions(&self) -> (Vec<FavSessionInfo>, Vec<FavSessionInfo>) {
        let flush_sessions: Vec<FavSessionInfo> = self
            .flush_sessions
            .iter()
            .filter(|session| {
                if let Some(filter) = self.filter.clone() {
                    session.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        let fav_sessions: Vec<FavSessionInfo> = self
            .fav_sessions
            .iter()
            .filter(|session| {
                if let Some(filter) = self.filter.clone() {
                    session.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        (fav_sessions, flush_sessions)
    }

    pub fn render_navigation(&self, cols: usize, rows: usize) {
        let half_cols = cols / 2;

        println!(
            "{} {}",
            ">".cyan().bold(),
            if let Some(filter) = &self.filter {
                filter.dimmed().italic().to_string()
            } else {
                "(filter)".dimmed().italic().to_string()
            },
        );

        let favs_title = if self.mode == FavMode::NavigateFavs {
            format!("{}", "Favorites".bold().blue())
        } else {
            format!("{}", "Favorites".bold().dimmed())
        };

        print_text_with_coordinates(Text::new(favs_title), 0, 1, None, None);

        let sessions_space = rows.saturating_sub(3);
        let skip = if self.cursor > sessions_space.saturating_sub(1) {
            self.cursor.saturating_sub(sessions_space.saturating_sub(1))
        } else {
            0
        };

        for (i, session) in self
            .fav_sessions
            .iter()
            .filter(|session| {
                if let Some(filter) = self.filter.clone() {
                    session.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .skip({
                if self.mode == FavMode::NavigateFavs {
                    skip
                } else {
                    0
                }
            })
            .enumerate()
        {
            if i >= sessions_space {
                break;
            }
            let selected_idx = self.cursor.min(sessions_space - 1);
            let text = if self.mode == FavMode::NavigateFavs && selected_idx == i {
                let selected = format!("{} {}", ">".cyan(), session.name.clone());
                Text::new(selected).selected()
            } else {
                Text::new(session.name.clone())
            };

            print_text_with_coordinates(text, 0, 2 + i, None, None);
        }

        let flush_title = if self.mode == FavMode::NavigateFlush {
            format!("{}", "Flush".bold().blue())
        } else {
            format!("{}", "Flush".bold().dimmed())
        };
        print_text_with_coordinates(Text::new(flush_title), half_cols, 1, None, None);

        for (i, session) in self
            .flush_sessions
            .iter()
            .filter(|session| {
                if let Some(filter) = self.filter.clone() {
                    session.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .skip({
                if self.mode == FavMode::NavigateFlush {
                    skip
                } else {
                    0
                }
            })
            .enumerate()
        {
            if i >= sessions_space {
                break;
            }
            let selected_idx = self.cursor.min(sessions_space - 1);
            let text = if self.mode == FavMode::NavigateFlush && selected_idx == i {
                let selected = format!("{} {}", ">".cyan(), session.name.clone());
                Text::new(selected).selected()
            } else {
                Text::new(session.name.clone())
            };
            print_text_with_coordinates(text, half_cols, 2 + i, None, None);
        }

        if self.mode == FavMode::Filter {
            return;
        }

        let help_text = format!("{}", "Press '?' for help".dimmed().italic());
        print_text_with_coordinates(Text::new(help_text), 0, rows - 1, None, None);
    }
    pub fn render_help_commands(&self) {
        let modes = FavMode::variants();
        for mode in modes.iter() {
            if mode == &FavMode::NavigateFlush {
                continue;
            }
            println!("{}", mode.clone().dimmed().italic().red());
            let commands = mode.clone().get_commands();

            for command in commands.iter() {
                println!("  {} - {}", command.0.purple(), command.1);
            }
        }
    }
}

impl ZellijPlugin for Favs {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);
        subscribe(&[
            EventType::Key,
            EventType::SessionUpdate,
            EventType::CustomMessage,
        ]);
    }

    fn update(&mut self, event: zellij_tile::prelude::Event) -> bool {
        let mut render = false;
        match event {
            Event::Key(key) => {
                render = self.match_key(&key.bare_key);
            }
            Event::SessionUpdate(sessions_info, resurrectable_session_list) => {
                let mut current_sessions: Vec<FavSessionInfo> = sessions_info
                    .iter()
                    .map(|session_info| FavSessionInfo {
                        name: session_info.name.clone(),
                        is_active: true,
                    })
                    .collect();

                current_sessions.extend(resurrectable_session_list.iter().map(|session_info| {
                    FavSessionInfo {
                        name: session_info.0.clone(),
                        is_active: false,
                    }
                }));

                let fav_sessions_json: Vec<String> =
                    serde_json::from_reader(File::open(FAVS_PATH_TMP).unwrap()).unwrap();

                eprintln!("fav_sessions_json: {:?}", fav_sessions_json);

                let (fav_sessions, flush_sessions): (Vec<FavSessionInfo>, Vec<FavSessionInfo>) =
                    current_sessions
                        .iter()
                        .cloned()
                        .partition(|current| fav_sessions_json.contains(&current.name));

                self.fav_sessions = fav_sessions;
                self.flush_sessions = flush_sessions;

                match self.mode {
                    FavMode::NavigateFavs => {
                        self.cursor = self.cursor.min(self.fav_sessions.len());
                    }
                    FavMode::NavigateFlush => {
                        self.cursor = self.cursor.min(self.flush_sessions.len());
                    }
                    _ => {}
                }

                self.commit_fav_changes();

                render = true;
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
    }
}
