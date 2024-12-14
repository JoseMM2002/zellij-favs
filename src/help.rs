use zellij_tile::prelude::BareKey;

use crate::{favs::Favs, favs_mode::FavMode};

pub fn match_help_keys(ctx: &mut Favs, key: &BareKey) -> bool {
    match key {
        BareKey::Esc | BareKey::Char('q') => {
            ctx.mode = FavMode::NavigateFavs;
        }
        _ => {}
    }
    true
}
