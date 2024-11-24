use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum FavMode {
    #[default]
    NavigateFavs,
    NavigateFlush,
    Filter,
}

impl FavMode {
    pub fn get_commands(self) -> Vec<String> {
        match self {
            FavMode::Filter => {
                vec![
                    format!(
                        "{} {}",
                        "<Enter>".bold().purple(),
                        "- Use filter".bold().yellow()
                    ),
                    format!(
                        "{} {}",
                        "<Left>".bold().purple(),
                        "- Go to Favs".bold().yellow()
                    ),
                    format!(
                        "{} {}",
                        "<Right>".bold().purple(),
                        "- Go to Flush".bold().yellow()
                    ),
                ]
            }
            FavMode::NavigateFavs => vec![
                format!(
                    "{} {}",
                    "<Enter>".bold().purple(),
                    "- Open session".bold().yellow()
                ),
                format!(
                    "{} {}",
                    "<Space>".bold().purple(),
                    "- Move session to Flush".bold().yellow()
                ),
                format!(
                    "{} {}",
                    "<Tab>".bold().purple(),
                    "- Navigate Flush items".bold().yellow()
                ),
                format!("{} {}", "/".bold().purple(), " - Filter".bold().yellow()),
            ],
            FavMode::NavigateFlush => vec![
                format!(
                    "{} {}",
                    "<Enter>".bold().purple(),
                    "- Open session".bold().yellow()
                ),
                format!(
                    "{} {}",
                    "<Space>".bold().purple(),
                    "- Move session to Favs".bold().yellow()
                ),
                format!(
                    "{} {}",
                    "<Tab>".bold().purple(),
                    "- Navigate Favs items".bold().yellow()
                ),
                format!("{} {}", "/".bold().purple(), "- Filter".bold().yellow()),
            ],
        }
    }
}
