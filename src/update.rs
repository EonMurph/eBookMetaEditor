use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};

use crate::model::Page;

pub enum EventMessage {
    Start,
    Quit,
}

pub fn update(model: &mut crate::model::Model, msg: EventMessage) {
    match msg {
        EventMessage::Quit => model.running = false,
        EventMessage::Start => model.current_screen = Page::SeriesData,
    }
}

pub fn handle_event(_: &crate::model::Model) -> color_eyre::Result<Option<EventMessage>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }

    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<EventMessage> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(EventMessage::Quit),
        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
            Some(EventMessage::Quit)
        }
        KeyCode::Enter => Some(EventMessage::Start),
        _ => None,
    }
}
