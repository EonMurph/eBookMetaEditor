use std::{fs::read_dir, path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::model::{FileList, Model, Page};

pub(crate) enum Direction {
    Left,
    Right,
}
pub enum EventMessage {
    ChangePage(Direction),
    Quit,
    SetSeriesCounter(i8),
    NextFile,
    PreviousFile,
    SelectFile,
}

pub fn update(model: &mut Model, msg: EventMessage) {
    let current_idx = model.inputs.current_series_num;

    match msg {
        EventMessage::Quit => model.running = false,
        EventMessage::ChangePage(direction) => {
            model.current_page = match Page::VALUES[model.current_page] {
                Page::SeriesData => {
                    model.set_num_series();
                    if model.inputs.file_lists.is_empty() {
                        let mut files_list: Vec<PathBuf> = vec![PathBuf::from("./../")];
                        files_list.extend(
                            read_dir("./")
                                .unwrap()
                                .filter_map(|entry| entry.ok())
                                .map(|entry| entry.path()),
                        );
                        for _ in 0..model.inputs.series_num {
                            model
                                .inputs
                                .file_lists
                                .push(FileList::from_iter(files_list.clone().into_iter()));
                        }
                    }
                    match direction {
                        Direction::Left => model.current_page.saturating_sub(1),
                        Direction::Right => {
                            model.current_page.saturating_add(1) % Page::VALUES.len()
                        }
                    }
                }
                Page::FileSelection => match direction {
                    Direction::Left => {
                        if model.inputs.current_series_num == 0 {
                            model.current_page.saturating_sub(1)
                        } else {
                            model.inputs.current_series_num -= 1;
                            model.current_page
                        }
                    }
                    Direction::Right => {
                        model.inputs.current_series_num += 1;

                        if model.inputs.current_series_num < model.inputs.series_num as usize {
                            model.current_page
                        } else {
                            model.inputs.current_series_num = 0;
                            0
                        }
                    }
                },
                _ => match direction {
                    Direction::Left => model.current_page.saturating_sub(1),
                    Direction::Right => model.current_page.saturating_add(1) % Page::VALUES.len(),
                },
            }
        }
        EventMessage::SetSeriesCounter(s) => {
            // Add or subtract 1 from num series and then clamp to 0 and 127
            model.inputs.series_num = model.inputs.series_num.saturating_add(s);
            model.inputs.series_num = model.inputs.series_num.clamp(1, i8::MAX);
        }
        EventMessage::NextFile => {
            model.inputs.file_lists[current_idx].state.next();
        }
        EventMessage::PreviousFile => {
            model.inputs.file_lists[current_idx].state.previous();
        }
        EventMessage::SelectFile => {
            let file_list = &mut model.inputs.file_lists[current_idx];
            let state = &file_list.state;
            if let Some(selected_idx) = state.selected {
                if file_list.selected.contains(&selected_idx) {
                    file_list.selected.remove(&selected_idx);
                } else {
                    file_list.selected.insert(selected_idx);
                }
            }
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
        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::ChangePage(Direction::Right))
        }
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::ChangePage(Direction::Left))
        }
        // Set page specific keybinds
        _ => match Page::VALUES[model.current_page] {
            Page::SeriesData => match key.code {
                KeyCode::Left => Some(EventMessage::SetSeriesCounter(-1)),
                KeyCode::Right => Some(EventMessage::SetSeriesCounter(1)),
                _ => None,
            },
            Page::FileSelection => match key.code {
                KeyCode::Down => Some(EventMessage::NextFile),
                KeyCode::Up => Some(EventMessage::PreviousFile),
                KeyCode::Tab => Some(EventMessage::SelectFile),
                _ => None,
            },
            _ => None,
        },
    }
}
