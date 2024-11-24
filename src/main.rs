use std::usize;
use zellij_favs::favs::Favs;
use zellij_favs::worker::FavSynchronizer;
use zellij_tile::prelude::*;
use zellij_tile::{register_plugin, register_worker, ZellijPlugin};

register_plugin!(Favs);
register_worker!(FavSynchronizer, fav_synchroniser, FAV_SYNCHRONIZER);
