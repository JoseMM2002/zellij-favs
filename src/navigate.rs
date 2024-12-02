use zellij_tile::{
    prelude::BareKey,
    shim::{close_self, delete_dead_session, kill_sessions, switch_session},
};

use crate::{favs::Favs, favs_mode::FavMode};

pub fn match_navigation_keys(ctx: &mut Favs, key: &BareKey) -> bool {
    let (fav_sessions, flush_sessions) = ctx.get_filtered_sessions();

    let arr_length = if ctx.mode == FavMode::NavigateFavs {
        fav_sessions.len()
    } else {
        flush_sessions.len()
    };
    match key {
        BareKey::Char('h') | BareKey::Left => {
            ctx.mode = FavMode::NavigateFavs;
            ctx.cursor = if ctx.cursor < fav_sessions.len() {
                ctx.cursor
            } else {
                fav_sessions.len() - 1
            };
        }
        BareKey::Char('j') | BareKey::Down => {
            if ctx.cursor + 1 < arr_length {
                ctx.cursor += 1;
            }
        }
        BareKey::Char('k') | BareKey::Up => {
            if ctx.cursor > 0 {
                ctx.cursor -= 1;
            }
        }
        BareKey::Char('l') | BareKey::Right => {
            ctx.mode = FavMode::NavigateFlush;
            ctx.cursor = if ctx.cursor < flush_sessions.len() {
                ctx.cursor
            } else {
                flush_sessions.len() - 1
            };
        }
        BareKey::Char('f') => {
            let sessions_to_delete: Vec<String> = flush_sessions
                .iter()
                .filter(|session| session.is_active)
                .map(|session| session.name.clone())
                .collect();

            kill_sessions(&sessions_to_delete);

            for session in flush_sessions.iter().filter(|session| session.is_active) {
                delete_dead_session(&session.name);
            }

            ctx.flush_sessions.retain(|session| {
                flush_sessions
                    .iter()
                    .find(|s| s.name == session.name)
                    .is_none()
            });
            ctx.commit_fav_changes();
        }
        BareKey::Char('/') => {
            ctx.mode = FavMode::Filter;
            ctx.filter = Some(String::new());
        }
        BareKey::Char(' ') => {
            if ctx.mode == FavMode::NavigateFavs {
                if fav_sessions.is_empty() {
                    return false;
                }

                let session = fav_sessions[ctx.cursor].clone();
                let session_idx = ctx
                    .fav_sessions
                    .iter()
                    .position(|s| s.name == session.name)
                    .unwrap();

                ctx.fav_sessions.remove(session_idx);
                ctx.flush_sessions.push(session);
                if ctx.cursor == ctx.fav_sessions.len() && !ctx.fav_sessions.is_empty() {
                    ctx.cursor -= 1;
                }
            } else {
                if flush_sessions.is_empty() {
                    return false;
                }

                let session = flush_sessions[ctx.cursor].clone();
                let session_idx = ctx
                    .flush_sessions
                    .iter()
                    .position(|s| s.name == session.name)
                    .unwrap();

                ctx.flush_sessions.remove(session_idx);
                ctx.fav_sessions.push(session);
                if ctx.cursor == ctx.flush_sessions.len() && !ctx.flush_sessions.is_empty() {
                    ctx.cursor -= 1;
                }
            }
            ctx.commit_fav_changes();
        }
        BareKey::Tab => {
            if ctx.mode == FavMode::NavigateFavs {
                ctx.mode = FavMode::NavigateFlush;
                ctx.cursor = ctx.cursor.min(ctx.flush_sessions.len());
            } else {
                ctx.mode = FavMode::NavigateFavs;
                ctx.cursor = ctx.cursor.min(ctx.fav_sessions.len());
            }
            ctx.cursor = 0;
        }
        BareKey::Enter => {
            let session = if ctx.mode == FavMode::NavigateFavs {
                ctx.fav_sessions[ctx.cursor].clone()
            } else {
                ctx.flush_sessions[ctx.cursor].clone()
            };
            switch_session(Some(session.name.as_str()));
            close_self();
        }
        BareKey::Esc => {
            close_self();
        }
        _ => return false,
    };
    true
}
