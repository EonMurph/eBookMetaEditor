use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::model::{Model, Page};

pub enum EventMessage {
    NextPage,
    Quit,
    SetSeriesCounter(i8),
}

pub fn update(model: &mut Model, msg: EventMessage) {
    match msg {
        EventMessage::Quit => model.running = false,
        EventMessage::NextPage => {
            model.current_page = match model.current_page {
                Page::Home => Page::SeriesData,
                Page::SeriesData => {
                    model.set_num_series();
                    Page::Home
                }
                _ => Page::Home,
            }
        }
        EventMessage::SetSeriesCounter(s) => {
            // Add or subtract 1 from num series and then clamp to 0 and 127
            model.inputs.series_num = model.inputs.series_num.saturating_add(s);
            model.inputs.series_num = model.inputs.series_num.clamp(0, i8::MAX);
        }
    }
}

pub fn handle_event(model: &Model) -> color_eyre::Result<Option<EventMessage>> {
    // Wait up to 250ms for an event
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(model, key));
            }
        }
    }

    Ok(None)
}

fn handle_key(model: &Model, key: event::KeyEvent) -> Option<EventMessage> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(EventMessage::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::Quit)
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
            Some(EventMessage::NextPage)
        }
        // Set page specific keybinds
        _ => match model.current_page {
            Page::SeriesData => match key.code {
                KeyCode::Left => Some(EventMessage::SetSeriesCounter(-1)),
                KeyCode::Right => Some(EventMessage::SetSeriesCounter(1)),
                _ => None,
            },
            _ => None,
        },
    }
}
