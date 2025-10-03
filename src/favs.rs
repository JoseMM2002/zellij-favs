use std::collections::BTreeMap;

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

pub struct Favs {
    pub fav_sessions: Vec<FavSessionInfo>,
    pub flush_sessions: Vec<FavSessionInfo>,
    pub cursor: usize,
    pub mode: FavMode,
    pub current_column: Option<FavMode>,
    pub filter: Option<String>,
    pub has_loaded: bool,
    pub cache_dir: String,
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FavsJson {
    pub favs: Vec<FavSessionInfo>,
    pub flush: Vec<FavSessionInfo>,
}

impl Into<FavsJson> for &Favs {
    fn into(self) -> FavsJson {
        FavsJson {
            favs: self.fav_sessions.clone(),
            flush: self.flush_sessions.clone(),
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
    pub fn get_mut_filtered_sessions(
        &mut self,
    ) -> (Vec<&mut FavSessionInfo>, Vec<&mut FavSessionInfo>) {
        if let Some(filter) = self.filter.clone() {
            return (
                self.fav_sessions
                    .iter_mut()
                    .filter(|session| session.name.to_lowercase().contains(&filter.to_lowercase()))
                    .collect(),
                self.flush_sessions
                    .iter_mut()
                    .filter(|session| session.name.to_lowercase().contains(&filter.to_lowercase()))
                    .collect(),
            );
        }

        (
            self.fav_sessions.iter_mut().collect(),
            self.flush_sessions.iter_mut().collect(),
        )
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
            let assigned_number = if let Some(assigned) = session.assigned_number {
                format!(" ({})", assigned).dimmed().to_string()
            } else {
                "".to_string()
            };
            let counts = format!(" ({} tabs, {} panes)", session.tabs, session.panes);
            let text = if self.mode == FavMode::NavigateFavs && selected_idx == i {
                let selected = format!(
                    "> {}{}{}",
                    session.name.clone().underline(),
                    assigned_number,
                    counts.dimmed()
                );
                Text::new(selected)
            } else if self.mode == FavMode::AssignNumber
                && self.current_column == Some(FavMode::NavigateFavs)
                && selected_idx == i
            {
                let editing_access_text =
                    format!("> {} {}", session.name.clone(), "(0-9)".dimmed());
                Text::new(editing_access_text)
            } else {
                Text::new(format!(
                    "{}{}{}",
                    session.name.clone(),
                    assigned_number,
                    counts.dimmed()
                ))
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
            let assigned_number = if let Some(assigned) = session.assigned_number {
                format!(" ({})", assigned).dimmed().to_string()
            } else {
                "".to_string()
            };
            let counts = format!(" ({} tabs, {} panes)", session.tabs, session.panes);
            let text = if self.mode == FavMode::NavigateFlush && selected_idx == i {
                let selected = format!(
                    "> {}{}{}",
                    session.name.clone().underline(),
                    assigned_number,
                    counts.dimmed()
                );
                Text::new(selected)
            } else if self.mode == FavMode::AssignNumber
                && self.current_column == Some(FavMode::NavigateFlush)
                && selected_idx == i
            {
                let editing_access_text = format!("> {} ({})", session.name.clone(), "0-9");
                Text::new(editing_access_text)
            } else {
                Text::new(format!(
                    "{}{}{}",
                    session.name.clone(),
                    assigned_number,
                    counts.dimmed()
                ))
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
}

impl ZellijPlugin for Favs {
    fn load(&mut self, configuration: std::collections::BTreeMap<String, String>) {
        if let Some(cache_dir) = configuration.get("cache_dir") {
            self.cache_dir = cache_dir.to_string();
        }
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::Key,
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
            Event::SessionUpdate(sessions_info, resurrectable_session_list) => {
                if !self.has_loaded {
                    self.load_cache();
                }
                let mut all_sessions: Vec<(&String, usize, usize, bool)> = sessions_info
                    .iter()
                    .map(|s| {
                        (
                            &s.name,
                            s.tabs.len(),
                            s.panes.panes.values().map(|v| v.len()).sum(),
                            true,
                        )
                    })
                    .collect();
                all_sessions.extend(
                    resurrectable_session_list
                        .iter()
                        .map(|s| (&s.0, 0, 0, false)),
                );

                let mut fav_sessions: Vec<FavSessionInfo> = vec![];
                let mut flush_sessions: Vec<FavSessionInfo> = vec![];

                for (session_name, tabs, panes, is_active) in all_sessions {
                    if let Some(fav_session) =
                        self.fav_sessions.iter().find(|s| &s.name == session_name)
                    {
                        fav_sessions.push(FavSessionInfo {
                            tabs,
                            panes,
                            is_active,
                            ..fav_session.clone()
                        });
                    } else if let Some(flush_session) =
                        self.flush_sessions.iter().find(|s| &s.name == session_name)
                    {
                        flush_sessions.push(FavSessionInfo {
                            tabs,
                            panes,
                            is_active,
                            ..flush_session.clone()
                        });
                    } else {
                        flush_sessions.push(FavSessionInfo {
                            name: session_name.to_string(),
                            is_active,
                            assigned_number: None,
                            tabs,
                            panes,
                        });
                    }
                }

                if self.fav_sessions != fav_sessions
                    || self.flush_sessions != flush_sessions && self.has_loaded
                {
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
            }
            Event::RunCommandResult(exit_code, stdout, _stderr, context) => {
                if exit_code.is_some() && exit_code != Some(0) {
                    self.has_loaded = true;
                    return true;
                }
                if let Some(command_type) = context.get(FavsCommandType::get_command_key().as_str())
                {
                    let command_type_enum: FavsCommandType = command_type.into();
                    if let FavsCommandType::ReadCache = command_type_enum {
                        if let Ok(json_string) = String::from_utf8(stdout) {
                            if let Ok(sessions) = serde_json::from_str::<FavsJson>(&json_string) {
                                self.fav_sessions = sessions.favs;
                                self.flush_sessions = sessions.flush;
                            }
                        }
                        self.has_loaded = true;
                        render = true;
                    }
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
    }
}
