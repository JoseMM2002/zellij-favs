use std::{fs::File, io::Write};

use owo_colors::OwoColorize;
use zellij_tile::{
    prelude::*,
    shim::{
        delete_dead_session, kill_sessions, print_text_with_coordinates, request_permission,
        subscribe, switch_session, Text,
    },
    ZellijPlugin,
};

use crate::{
    favs_mode::FavMode, filter::match_filter_key, FavSessionInfo, FAVS_PATH_TMP, FAVS_TEMPLATE,
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
            FavMode::Filter => {
                match_filter_key(self, key);
            }
            _ => {
                let flush_sessions: Vec<&FavSessionInfo> = self
                    .flush_sessions
                    .iter()
                    .filter(|session| {
                        if let Some(filter) = self.filter.clone() {
                            session.name.to_lowercase().contains(&filter.to_lowercase())
                        } else {
                            true
                        }
                    })
                    .collect();
                let fav_sessions: Vec<&FavSessionInfo> = self
                    .fav_sessions
                    .iter()
                    .filter(|session| {
                        if let Some(filter) = self.filter.clone() {
                            session.name.to_lowercase().contains(&filter.to_lowercase())
                        } else {
                            true
                        }
                    })
                    .collect();

                let arr_length = if self.mode == FavMode::NavigateFavs {
                    fav_sessions.len()
                } else {
                    flush_sessions.len()
                };
                match key {
                    BareKey::Char('h') | BareKey::Left => {
                        self.mode = FavMode::NavigateFavs;
                        self.cursor = if self.cursor < fav_sessions.len() {
                            self.cursor
                        } else {
                            fav_sessions.len() - 1
                        };
                    }
                    BareKey::Char('j') | BareKey::Down => {
                        if self.cursor + 1 < arr_length {
                            self.cursor += 1;
                        }
                    }
                    BareKey::Char('k') | BareKey::Up => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    BareKey::Char('l') | BareKey::Right => {
                        self.mode = FavMode::NavigateFlush;
                        self.cursor = if self.cursor < flush_sessions.len() {
                            self.cursor
                        } else {
                            flush_sessions.len() - 1
                        };
                    }
                    BareKey::Char('f') => {
                        let sessions_to_delete: Vec<String> = self
                            .flush_sessions
                            .iter()
                            .filter(|session| session.is_active)
                            .map(|session| session.name.clone())
                            .collect();

                        kill_sessions(&sessions_to_delete);

                        for session in self.flush_sessions.iter() {
                            delete_dead_session(&session.name);
                        }

                        self.flush_sessions = vec![];
                        self.commit_fav_changes();
                    }
                    BareKey::Char('/') => {
                        self.mode = FavMode::Filter;
                        self.filter = Some(String::new());
                    }
                    BareKey::Char(' ') => {
                        if self.mode == FavMode::NavigateFavs {
                            if fav_sessions.is_empty() {
                                return false;
                            }

                            let session = fav_sessions[self.cursor].clone();
                            let session_idx = self
                                .fav_sessions
                                .iter()
                                .position(|s| s.name == session.name)
                                .unwrap();

                            self.fav_sessions.remove(session_idx);
                            self.flush_sessions.push(session);
                            if self.cursor == self.fav_sessions.len()
                                && !self.fav_sessions.is_empty()
                            {
                                self.cursor -= 1;
                            }
                        } else {
                            if flush_sessions.is_empty() {
                                return false;
                            }

                            let session = flush_sessions[self.cursor].clone();
                            let session_idx = self
                                .flush_sessions
                                .iter()
                                .position(|s| s.name == session.name)
                                .unwrap();

                            self.flush_sessions.remove(session_idx);
                            self.fav_sessions.push(session);
                            if self.cursor == self.flush_sessions.len()
                                && !self.flush_sessions.is_empty()
                            {
                                self.cursor -= 1;
                            }
                        }
                        self.commit_fav_changes();
                    }
                    BareKey::Tab => {
                        if self.mode == FavMode::NavigateFavs {
                            self.mode = FavMode::NavigateFlush;
                            self.cursor = self.cursor.min(self.flush_sessions.len());
                        } else {
                            self.mode = FavMode::NavigateFavs;
                            self.cursor = self.cursor.min(self.fav_sessions.len());
                        }
                        self.cursor = 0;
                    }
                    BareKey::Enter => {
                        let session = if self.mode == FavMode::NavigateFavs {
                            self.fav_sessions[self.cursor].clone()
                        } else {
                            self.flush_sessions[self.cursor].clone()
                        };
                        switch_session(Some(session.name.as_str()));
                        close_self();
                    }
                    BareKey::Esc => {
                        close_self();
                    }
                    _ => return false,
                };
            }
        }
        true
    }
    fn commit_fav_changes(&self) {
        let favs_to_save: Vec<String> = self
            .fav_sessions
            .iter()
            .map(|session| session.name.clone())
            .collect();
        let json = serde_json::to_string(&favs_to_save).unwrap();

        let mut file = File::create(FAVS_PATH_TMP).unwrap();
        file.write_all(json.as_bytes()).unwrap();
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
                    FavMode::Filter => {}
                }

                self.commit_fav_changes();

                render = true;
            }
            _ => {}
        }

        render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
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
            .enumerate()
        {
            let text = if self.mode == FavMode::NavigateFavs && self.cursor == i {
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
            .enumerate()
        {
            let text = if self.mode == FavMode::NavigateFlush && self.cursor == i {
                let selected = format!("{} {}", ">".cyan(), session.name.clone());
                Text::new(selected).selected()
            } else {
                Text::new(session.name.clone())
            };
            print_text_with_coordinates(text, half_cols, 2 + i, None, None);
        }
    }
}
