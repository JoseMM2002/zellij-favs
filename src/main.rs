use owo_colors::OwoColorize;
use std::fs::File;
use std::io::Write;
use std::usize;

use serde::{Deserialize, Serialize};
use zellij_tile::prelude::*;
use zellij_tile::{register_plugin, shim::subscribe, ZellijPlugin};

const FAVS_TEMPLATE: &str = r#"[]"#;

const FAVS_PATH: &str = "favs.json";

#[derive(Default, Clone, Serialize, Deserialize)]
struct FavSessionInfo {
    name: String,
    is_active: bool,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
enum FavMode {
    #[default]
    NavigateFavs,
    NavigateFlush,
    Filter,
}

struct Favs {
    fav_sessions: Vec<FavSessionInfo>,
    flush_sessions: Vec<FavSessionInfo>,
    cursor: usize,
    mode: FavMode,
    filter: Option<String>,
}

impl Default for Favs {
    fn default() -> Self {
        if !std::path::Path::new(FAVS_PATH).exists() {
            let mut file = File::create(FAVS_PATH).unwrap();
            file.write_all(FAVS_TEMPLATE.as_bytes()).unwrap();
        }
        let fav_sessions_json: Vec<String> =
            serde_json::from_reader(File::open(FAVS_PATH).unwrap()).unwrap();

        Self {
            fav_sessions: fav_sessions_json
                .iter()
                .map(|name| FavSessionInfo {
                    name: name.to_string(),
                    is_active: false,
                })
                .collect(),
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
                if let Some(filter) = self.filter.as_mut() {
                    match key {
                        BareKey::Char(char) => {
                            filter.push(*char);
                        }
                        BareKey::Backspace => {
                            filter.pop();
                        }
                        BareKey::Insert | BareKey::Left => {
                            self.mode = FavMode::NavigateFavs;
                            self.cursor = 0;
                        }
                        BareKey::Right => {
                            self.mode = FavMode::NavigateFlush;
                            self.cursor = 0;
                        }
                        _ => return false,
                    }
                }
            }
            _ => {
                let arr_length = if self.mode == FavMode::NavigateFavs {
                    self.fav_sessions.len()
                } else {
                    self.flush_sessions.len()
                };
                match key {
                    BareKey::Char('j') => {
                        self.cursor = (self.cursor + 1).min(arr_length);
                    }
                    BareKey::Char('k') => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
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
                    }
                    BareKey::Char('r') => {
                        self.mode = FavMode::Filter;
                        self.filter = Some(String::new());
                    }
                    BareKey::Char(' ') => {
                        if self.mode == FavMode::NavigateFavs {
                            let session = self.fav_sessions.remove(self.cursor);
                            self.flush_sessions.push(session);
                        } else {
                            let session = self.fav_sessions.remove(self.cursor);
                            self.flush_sessions.push(session);
                        }
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
                    _ => return false,
                };
            }
        }
        true
    }
}

impl ZellijPlugin for Favs {
    fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::Key, EventType::SessionUpdate]);
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

                let (fav_sessions, flush_sessions): (Vec<FavSessionInfo>, Vec<FavSessionInfo>) =
                    current_sessions.iter().cloned().partition(|current| {
                        self.fav_sessions
                            .iter()
                            .any(|saved| saved.name == current.name)
                    });

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
            }
        );

        let favorites_title = "Favorites";
        let text = Text::new(favorites_title).color_range(0, 0..favorites_title.len());
        print_text_with_coordinates(text, 0, 1, None, None);

        for (i, session) in self.fav_sessions.iter().enumerate() {
            let text = if self.mode == FavMode::NavigateFavs && self.cursor == i {
                Text::new(session.name.clone()).selected()
            } else {
                Text::new(session.name.clone())
            };

            print_text_with_coordinates(text, 0, 2 + i, None, None);
        }

        let flush_title = "Flush";
        let text = Text::new(flush_title).color_range(0, 0..flush_title.len());
        print_text_with_coordinates(text, half_cols, 1, None, None);

        for (i, session) in self.flush_sessions.iter().enumerate() {
            let text = if self.mode == FavMode::NavigateFlush && self.cursor == i {
                Text::new(session.name.clone()).selected()
            } else {
                Text::new(session.name.clone())
            };
            print_text_with_coordinates(text, half_cols, 2 + i, None, None);
        }
    }
}

register_plugin!(Favs);
