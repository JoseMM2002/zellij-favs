use zellij_tile::prelude::BareKey;

use crate::{favs::Favs, favs_mode::FavMode};

pub fn match_filter_key(ctx: &mut Favs, key: &BareKey) -> bool {
    if let Some(filter) = ctx.filter.as_mut() {
        match key {
            BareKey::Char(char) => {
                filter.push(*char);
            }
            BareKey::Backspace => {
                filter.pop();
            }
            BareKey::Enter | BareKey::Left => {
                ctx.mode = FavMode::NavigateFavs;
                ctx.cursor = 0;
            }
            BareKey::Right | BareKey::Tab => {
                ctx.mode = FavMode::NavigateFlush;
                ctx.cursor = 0;
            }
            BareKey::Esc => {
                ctx.filter = None;
                ctx.mode = FavMode::NavigateFavs;
            }
            _ => return false,
        }
    }
    true
}
