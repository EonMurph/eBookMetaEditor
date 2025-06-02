use std::{collections::HashMap, fs::canonicalize, path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::model::{FileList, InputField, Model, Page};

/// Enum for holding direction for page changing event
pub(crate) enum Direction {
    Previous,
    Next,
}
/// Enum for holding possible app events
pub enum EventMessage {
    /// Change the current page
    ChangePage(Direction),
    /// Quit the app
    Quit,
    /// Change the number of series to edit
    SetSeriesCounter(i8),
    /// Go to the next file in selection page
    NextFile,
    /// Go to the previous file in selection page
    PreviousFile,
    /// Select file in selection page
    SelectFile,
    /// Change current directory in selection page
    ChangeDirectory(PathBuf),
    /// Change the input field being worked on
    ChangeField(Direction),
    /// Input text into the input field
    InputText(char),
    /// Remove text from the input field
    RemoveText,
}

/// Function for processing events
pub fn update(model: &mut Model, msg: EventMessage) {
    let current_idx = model.inputs.current_series_num;

    match msg {
        EventMessage::Quit => model.running = false,
        EventMessage::ChangePage(direction) => {
            model.current_page = match Page::VALUES[model.current_page] {
                Page::SeriesData => {
                    if model.inputs.series_num > model.inputs.file_lists.len() as i8 {
                        for _ in 0..(model.inputs.series_num - model.inputs.file_lists.len() as i8)
                        {
                            model.inputs.file_lists.push(FileList::from_iter(
                                model
                                    .get_current_file_list(PathBuf::from("./"))
                                    .clone()
                                    .into_iter(),
                            ));
                            model.inputs.field_values.push(HashMap::from([
                                (InputField::Author, String::from("Surname, Forename")),
                                (InputField::Series, String::from("Placeholder title")),
                                (
                                    InputField::Format,
                                    String::from("{series_name} ({position}) - {book_title}"),
                                ),
                            ]));
                        }
                    }
                    match direction {
                        Direction::Previous => model.current_page.saturating_sub(1),
                        Direction::Next => {
                            model.current_page.saturating_add(1) % Page::VALUES.len()
                        }
                    }
                }
                Page::FileSelection => match direction {
                    Direction::Previous => {
                        if current_idx == 0 {
                            model.current_page.saturating_sub(1)
                        } else {
                            model.inputs.current_series_num -= 1;
                            model.current_page.saturating_add(1)
                        }
                    }
                    Direction::Next => {
                        if !model.inputs.file_lists[current_idx].selected.is_empty() {
                            model.current_page.saturating_add(1)
                        } else {
                            model.current_page
                        }
                    }
                },
                Page::BookData => match direction {
                    Direction::Previous => model.current_page.saturating_sub(1),
                    Direction::Next => {
                        if model.inputs.current_series_num < model.inputs.series_num as usize - 1 {
                            model.inputs.current_series_num += 1;
                            model.current_page.saturating_sub(1)
                        } else {
                            model.current_page.saturating_add(1) % Page::VALUES.len()
                        }
                    }
                },
                _ => match direction {
                    Direction::Previous => model.current_page.saturating_sub(1),
                    Direction::Next => model.current_page.saturating_add(1) % Page::VALUES.len(),
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
                let file_name = &file_list.items[selected_idx];
                if file_name.is_file() {
                    if file_list.selected.contains(file_name) {
                        file_list.selected.retain(|value| value != file_name);
                    } else {
                        file_list.selected.push(file_name.to_owned());
                    }
                }
            }
        }
        EventMessage::ChangeDirectory(directory) => {
            if directory.is_dir() {
                let items = model.get_current_file_list(directory.clone());
                let file_list = &mut model.inputs.file_lists[current_idx];
                file_list.items = items;
                file_list.current_directory = canonicalize(directory).unwrap();
                file_list.state.selected = Some(0);
            }
        }
        EventMessage::ChangeField(direction) => match direction {
            Direction::Previous => {}
            Direction::Next => {
                model.inputs.currently_editing = match model.inputs.currently_editing {
                    InputField::Author => InputField::Series,
                    InputField::Series => InputField::Format,
                    InputField::Format => InputField::BookOrder,
                    InputField::BookOrder => InputField::Author,
                }
            }
        },
        EventMessage::InputText(char) => {
            if let Some(value) =
                model.inputs.field_values[current_idx].get_mut(&model.inputs.currently_editing)
            {
                value.push(char);
            }
        }
        EventMessage::RemoveText => {
            if let Some(value) =
                model.inputs.field_values[current_idx].get_mut(&model.inputs.currently_editing)
            {
                value.pop();
            }
        }
    }
}

/// Function for polling events and keybinds and returning related event
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

/// Function for processing key presses and returning related event
fn handle_key(model: &Model, key: event::KeyEvent) -> Option<EventMessage> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(EventMessage::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::Quit)
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::ChangePage(Direction::Next))
        }
        KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EventMessage::ChangePage(Direction::Previous))
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
                KeyCode::Right | KeyCode::Left => match key.code {
                    KeyCode::Right => {
                        if let Some(new_directory_index) = model.inputs.file_lists
                            [model.inputs.current_series_num]
                            .state
                            .selected
                        {
                            let new_directory = model.inputs.file_lists
                                [model.inputs.current_series_num]
                                .items[new_directory_index]
                                .clone();
                            Some(EventMessage::ChangeDirectory(new_directory))
                        } else {
                            None
                        }
                    }
                    KeyCode::Left => {
                        let current_directory = &model.inputs.file_lists
                            [model.inputs.current_series_num]
                            .current_directory;
                        current_directory
                            .parent()
                            .map(|parent| EventMessage::ChangeDirectory(parent.into()))
                    }
                    _ => None,
                },
                _ => None,
            },
            Page::BookData => match key.code {
                KeyCode::Tab => Some(EventMessage::ChangeField(Direction::Next)),
                KeyCode::Backspace => Some(EventMessage::RemoveText),
                KeyCode::Char(value) => Some(EventMessage::InputText(value)),
                _ => None,
            },
            _ => None,
        },
    }
}
