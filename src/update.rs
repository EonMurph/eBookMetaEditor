use std::{
    fs::{canonicalize, read_dir},
    path::PathBuf,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::model::{FileList, Model, Page};

/// Enum for holding direction for page changing event
pub(crate) enum Direction {
    Left,
    Right,
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
                        let directory_contents: Vec<PathBuf> = read_dir("./")
                            .unwrap()
                            .filter_map(|entry| entry.ok())
                            .map(|entry| canonicalize(entry.path()).unwrap())
                            .collect();
                        let mut files_list: Vec<PathBuf> = Vec::new();
                        {
                            let mut directories: Vec<PathBuf> = Vec::new();
                            let mut files: Vec<PathBuf> = Vec::new();
                            for entry in directory_contents {
                                if entry.is_dir() {
                                    directories.push(entry);
                                } else if entry.is_file() {
                                    files.push(entry);
                                }
                            }
                            directories.sort();
                            files.sort();
                            files_list.append(&mut directories);
                            files_list.append(&mut files);
                        }
                        for _ in 0..(model.inputs.series_num - model.inputs.file_lists.len() as i8)
                        {
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
                            model.current_page.saturating_add(1) % Page::VALUES.len()
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
                let file_name = &file_list.items[selected_idx];
                if file_name.is_file() {
                    if file_list.selected.contains(file_name) {
                        file_list.selected.remove(file_name);
                    } else {
                        file_list.selected.insert(file_name.to_owned());
                    }
                }
            }
        }
        EventMessage::ChangeDirectory(directory) => {
            if directory.is_dir() {
                let files_list = &mut model.inputs.file_lists[model.inputs.current_series_num];
                files_list.current_directory = directory;
                files_list.items = read_dir(&files_list.current_directory)
                    .unwrap()
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .collect();
                files_list.state.selected = Some(0);
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
            _ => None,
        },
    }
}
