use zellij_tile::prelude::BareKey;

use crate::{favs::Favs, favs_mode::FavMode};

pub fn match_assignation_keys(ctx: &mut Favs, key: &BareKey) -> bool {
    if let Some(column_mode) = ctx.current_column.clone() {
        match key {
            BareKey::Char(c) if c.is_ascii_digit() => {
                let index = c.to_digit(10).unwrap() as u8;
                let cursor = ctx.cursor;
                let (mut fav_sessions, mut flush_sessions) = ctx.get_mut_filtered_sessions();

                for session in fav_sessions.iter_mut() {
                    if let Some(assigned) = session.assigned_number {
                        if assigned == index {
                            session.assigned_number = None;
                        }
                    }
                }

                for session in flush_sessions.iter_mut() {
                    if let Some(assigned) = session.assigned_number {
                        if assigned == index {
                            session.assigned_number = None;
                        }
                    }
                }

                match column_mode {
                    FavMode::NavigateFavs => {
                        if let Some(session) = fav_sessions.get_mut(cursor) {
                            session.assigned_number = Some(index);
                        }
                    }
                    FavMode::NavigateFlush => {
                        if let Some(session) = flush_sessions.get_mut(cursor) {
                            session.assigned_number = Some(index);
                        }
                    }
                    _ => {}
                }
                ctx.mode = column_mode;
                ctx.current_column = None;
                ctx.commit_fav_changes();
            }
            BareKey::Esc => {
                ctx.mode = column_mode;
                ctx.current_column = None;
            }
            _ => return false,
        }
    }
    true
}
