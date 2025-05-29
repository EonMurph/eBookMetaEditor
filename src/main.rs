mod model;
mod tui;
mod update;
mod view;

use model::Model;
use update::{handle_event, update};
use view::View;

use cli_log::*;
use std::io;

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> color_eyre::Result<()> {
    init_cli_log!("ebook");
    tui::install_panic_hook();
    let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = tui::init_terminal()?;

    let mut model: Model = Model::new();
    while model.running {
        terminal.draw(|f| {
            if let Err(err) = View::draw(&mut model, f) {
                eprintln!("Error: {}", err);
            }
        })?;

        let current_msg = handle_event(&model)?;

        // If handle_event returned a message then update
        if let Some(msg) = current_msg {
            update(&mut model, msg);
        }
    }

    tui::restore_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}
