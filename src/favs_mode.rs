use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum FavMode {
    #[default]
    NavigateFavs,
    NavigateFlush,
    Filter,
    Help,
}

impl FavMode {
    pub fn get_commands(self) -> Vec<(&'static str, &'static str)> {
        match self {
            FavMode::Filter => {
                vec![
                    ("<Enter> | <Tab>", "Use filter"),
                    ("<Esc>", "Close filter"),
                    ("<Backspace>", "Delete last character"),
                    ("<Left>", "Go to Favs"),
                    ("<Right>", "Go to Flush"),
                    ("<Char>", "Type character to filter"),
                ]
            }
            FavMode::NavigateFavs | FavMode::NavigateFlush => vec![
                ("<Enter>", "Open session"),
                ("<Space>", "Move session to Flush/Favorites"),
                ("<Tab>", "Navigate Flush/Favorites items"),
                ("↑k | ↓j", "Move cursor"),
                ("/", "Filter"),
                ("?", "Help"),
                ("<Esc> | 'q'", "Close"),
            ],
            FavMode::Help => vec![("<Esc> | 'q'", "Close help"), ("↑k | ↓j", "Move cursor")],
        }
    }
    pub fn variants() -> Vec<Self> {
        vec![
            FavMode::NavigateFavs,
            FavMode::NavigateFlush,
            FavMode::Filter,
            FavMode::Help,
        ]
    }
    pub fn total_commands() -> usize {
        let modes = FavMode::variants();
        let mut total = 0;
        for mode in modes {
            total += mode.get_commands().len();
        }
        total
    }
}

impl Display for FavMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FavMode::NavigateFavs | FavMode::NavigateFlush => write!(f, "Navigate"),
            FavMode::Filter => write!(f, "Filter"),
            FavMode::Help => write!(f, "Help"),
        }
    }
}
