use zellij_tile::prelude::get_session_list;

use super::Favs;
use crate::{favs_mode::FavMode, FavSessionInfo};

impl Favs {
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

    pub fn refresh_session_list(&mut self) -> bool {
        let (sessions_info, resurrectable_session_list) = match get_session_list() {
            Ok(snapshot) => (snapshot.live_sessions, snapshot.resurrectable_sessions),
            Err(err) => {
                self.error = Some(format!("Failed to get session list: {}", err));
                return true;
            }
        };

        self.error = None;

        let mut all_sessions: Vec<(&String, usize, usize, bool)> = sessions_info
            .iter()
            .map(|s| {
                (
                    &s.name,
                    s.tabs.len(),
                    s.panes
                        .panes
                        .values()
                        .flat_map(|v| v.iter())
                        .filter(|pane| !pane.is_plugin)
                        .count(),
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
            if let Some(fav_session) = self.fav_sessions.iter().find(|s| &s.name == session_name) {
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
            || (self.flush_sessions != flush_sessions && self.has_loaded)
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
            return true;
        }

        false
    }
}
