mod model;
mod tui;
mod update;
mod view;

use model::Model;
use update::{handle_event, update};
use view::View;

use std::io;

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = tui::init_terminal()?;

    let mut model: Model = Model::new();
    while model.running {
        terminal.draw(|f| View::draw(&model, f))?;

        let current_msg = handle_event(&model)?;

        if let Some(msg) = current_msg {
            update(&mut model, msg);
        }
    }

    tui::restore_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}
