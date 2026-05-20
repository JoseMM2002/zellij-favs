use owo_colors::OwoColorize;
use zellij_tile::shim::{print_text_with_coordinates, Text};

use super::Favs;
use crate::favs_mode::FavMode;

impl Favs {
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

            let counters = if self.display_tab_panes {
                if session.is_active {
                    format!(" ({} tabs, {} panes)", session.tabs, session.panes)
                } else {
                    " (resurrect)".to_string()
                }
            } else {
                "".to_string()
            };

            let text = if self.mode == FavMode::NavigateFavs && selected_idx == i {
                let selected = format!(
                    "> {}{}{}",
                    session.name.clone().underline(),
                    assigned_number,
                    counters.dimmed()
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
                    counters.dimmed()
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

            let counts = if self.display_tab_panes {
                format!(" ({} tabs, {} panes)", session.tabs, session.panes)
            } else {
                "".to_string()
            };

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
}
