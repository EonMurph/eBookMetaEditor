mod model;
mod tui;
mod update;
mod view;

use model::Model;
use update::{EventMessage, handle_event, update};
use view::View;

use std::io;

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = tui::init_terminal()?;

    let mut model: Model = Model::new();
    while model.running {
        terminal.draw(|f| View::draw(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        update(&mut model, current_msg.unwrap());
    }

    tui::restore_terminal()?;
    terminal.show_cursor()?;

    Ok(())
}
