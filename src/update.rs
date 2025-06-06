use std::{cmp::Ordering, collections::HashMap, fs::canonicalize, path::PathBuf, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::widgets::TableState;

use crate::model::{FileList, InputField, Model, Page};

/// Enum for holding direction for page changing event
pub(crate) enum Direction {
    Previous,
    Next,
}
pub(crate) enum TableDirection {
    PreviousCol,
    NextCol,
    PreviousRow,
    NextRow,
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
    ChangeField,
    /// Change the input field within the file table
    ChangeTableField(TableDirection),
    /// Input text into the input field
    InputText(char),
    /// Remove text from the input field
    RemoveText,
    /// Move the books position one down or up in the series
    SwapBook(Direction),
    /// Change the index of the book in the series
    ChangeBookPosition(usize),
}

/// Function for processing events
pub fn update(model: &mut Model, msg: EventMessage) {
    let current_series = model.inputs.current_series_num;

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
                            if model.inputs.series_num > model.inputs.field_values.len() as i8 {
                                model
                                    .inputs
                                    .field_values
                                    .push(HashMap::from([(InputField::BookTitle, Vec::new())]));
                            }
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
                        if current_series == 0 {
                            model.current_page.saturating_sub(1)
                        } else {
                            model.inputs.current_series_num -= 1;
                            model.current_page.saturating_add(1)
                        }
                    }
                    Direction::Next => {
                        if !model.inputs.file_lists[current_series].selected.is_empty() {
                            if model.inputs.field_values[current_series].len() == 1 {
                                model.inputs.field_values[current_series].insert(
                                    InputField::Series,
                                    vec![String::from("Placeholder title")],
                                );
                                model.inputs.field_values[current_series].insert(
                                    InputField::Format,
                                    // vec![String::from("${series_name} (${position}) - ${title}")],
                                    vec![String::from("${series} (${position}) - ${title}")],
                                );
                                model.inputs.file_table_states.push(TableState::new());
                            }
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
            model.inputs.file_lists[current_series].state.next();
        }
        EventMessage::PreviousFile => {
            model.inputs.file_lists[current_series].state.previous();
        }
        EventMessage::SelectFile => {
            let file_list = &mut model.inputs.file_lists[current_series];
            let state = &mut file_list.state;
            if let Some(selected_idx) = state.selected {
                let file_name = &file_list.items[selected_idx];
                if file_name.is_file() {
                    if file_list.selected.contains(file_name) {
                        for i in 0..file_list.selected.len() {
                            if &file_list.selected[i] == file_name {
                                file_list.selected.remove(i);
                                if let Some(book_titles) = model.inputs.field_values[current_series]
                                    .get_mut(&InputField::BookTitle)
                                {
                                    book_titles.remove(i);
                                }
                                break;
                            }
                        }
                    } else {
                        file_list.selected.push(file_name.to_owned());
                        if let Some(book_titles) = model.inputs.field_values[current_series]
                            .get_mut(&InputField::BookTitle)
                        {
                            book_titles.push(
                                EpubDoc::new(file_name)
                                    .unwrap()
                                    .metadata
                                    .get("title")
                                    .unwrap()[0]
                                    .to_owned(),
                            )
                        }
                    }
                }
            }
            state.next();
        }
        EventMessage::ChangeDirectory(directory) => {
            if directory.is_dir() {
                let items = model.get_current_file_list(directory.clone());
                let file_list = &mut model.inputs.file_lists[current_series];
                file_list.items = items;
                file_list.current_directory = canonicalize(directory).unwrap();
                file_list.state.selected = Some(0);
            }
        }
        EventMessage::ChangeField => {
            model.inputs.currently_editing = match model.inputs.currently_editing {
                InputField::Series => InputField::Format,
                InputField::Format => InputField::BookOrder,
                InputField::BookOrder => InputField::Series,
                InputField::BookTitle => InputField::BookTitle,
            }
        },
        EventMessage::InputText(char) => {
            if model.inputs.currently_editing == InputField::BookOrder {
                let (current_row, current_cell) = model.inputs.file_table_states[current_series]
                    .selected_cell()
                    .unwrap_or_default();
                if current_cell == 1 {
                    if let Some(book_titles) =
                        model.inputs.field_values[current_series].get_mut(&InputField::BookTitle)
                    {
                        let book_title = &mut book_titles[current_row];
                        book_title.push(char);
                    }
                }
            }
            if let Some(value) =
                model.inputs.field_values[current_series].get_mut(&model.inputs.currently_editing)
            {
                let value = &mut value[0];
                value.push(char);
            }
        }
        EventMessage::RemoveText => {
            if model.inputs.currently_editing == InputField::BookOrder {
                let (current_row, current_cell) = model.inputs.file_table_states[current_series]
                    .selected_cell()
                    .unwrap_or_default();
                if current_cell == 1 {
                    if let Some(book_titles) =
                        model.inputs.field_values[current_series].get_mut(&InputField::BookTitle)
                    {
                        let book_title = &mut book_titles[current_row];
                        book_title.pop();
                    }
                }
            }
            if let Some(value) =
                model.inputs.field_values[current_series].get_mut(&model.inputs.currently_editing)
            {
                value[0].pop();
            }
        }
        EventMessage::ChangeTableField(direction) => {
            let table_state = &mut model.inputs.file_table_states[current_series];
            let row_selected = table_state.selected().is_some();
            match direction {
                TableDirection::PreviousCol if row_selected => {
                    table_state.select_previous_column();
                }
                TableDirection::NextCol if row_selected => {
                    table_state.select_next_column();
                }
                TableDirection::PreviousRow => {
                    table_state.select_previous();
                }
                TableDirection::NextRow => {
                    table_state.select_next();
                }
                _ => {}
            }
        }
        EventMessage::SwapBook(direction) => {
            let table_state = &mut model.inputs.file_table_states[current_series];
            if let Some(selected_row) = table_state.selected() {
                let book_list = &mut model.inputs.file_lists[current_series].selected;
                let book_titles = model.inputs.field_values[current_series]
                    .get_mut(&InputField::BookTitle)
                    .unwrap();
                match direction {
                    Direction::Next => {
                        if selected_row < book_list.len() - 1 {
                            book_list.swap(selected_row, selected_row + 1);
                            book_titles.swap(selected_row, selected_row + 1);
                            table_state.select_next();
                        }
                    }
                    Direction::Previous => {
                        if selected_row > 0 {
                            book_list.swap(selected_row, selected_row - 1);
                            book_titles.swap(selected_row, selected_row - 1);
                            table_state.select_previous();
                        }
                    }
                }
            }
        }
        EventMessage::ChangeBookPosition(new_index) => {
            if let Some(current_index) = model.inputs.file_table_states[current_series].selected() {
                let book_list = &mut model.inputs.file_lists[current_series].selected;
                let book_titles = model.inputs.field_values[current_series]
                    .get_mut(&InputField::BookTitle)
                    .unwrap();
                let num_books = book_list.len();
                if (0..num_books).contains(&new_index) {
                    match new_index.cmp(&current_index) {
                        Ordering::Less => {
                            for i in ((new_index + 1)..=current_index).rev() {
                                book_list.swap(i, i - 1);
                                book_titles.swap(i, i - 1);
                            }
                        }
                        Ordering::Equal => {}
                        Ordering::Greater => {
                            for i in current_index..new_index {
                                book_list.swap(i, i + 1);
                                book_titles.swap(i, i + 1);
                            }
                        }
                    }
                    model.inputs.file_table_states[current_series].select(Some(new_index));
                }
            };
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
                KeyCode::Tab => Some(EventMessage::ChangeField),
                _ => {
                    if model.inputs.currently_editing == InputField::BookOrder {
                        let current_col = model.inputs.file_table_states
                            [model.inputs.current_series_num]
                            .selected_column()
                            .unwrap_or_default();

                        match key.code {
                            KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                Some(EventMessage::SwapBook(Direction::Previous))
                            }
                            KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                Some(EventMessage::SwapBook(Direction::Next))
                            }
                            KeyCode::Right => {
                                Some(EventMessage::ChangeTableField(TableDirection::NextCol))
                            }
                            KeyCode::Left => {
                                Some(EventMessage::ChangeTableField(TableDirection::PreviousCol))
                            }
                            KeyCode::Up => {
                                Some(EventMessage::ChangeTableField(TableDirection::PreviousRow))
                            }
                            KeyCode::Down => {
                                Some(EventMessage::ChangeTableField(TableDirection::NextRow))
                            }
                            KeyCode::Char(value) => {
                                if value.is_ascii_digit() && current_col != 1 {
                                    Some(EventMessage::ChangeBookPosition(
                                        (value.to_digit(10).unwrap() as usize).saturating_sub(1),
                                    ))
                                } else if current_col == 1 {
                                    Some(EventMessage::InputText(value))
                                } else {
                                    None
                                }
                            }
                            KeyCode::Backspace if current_col == 1 => {
                                Some(EventMessage::RemoveText)
                            }
                            _ => None,
                        }
                    } else {
                        match key.code {
                            KeyCode::Backspace => Some(EventMessage::RemoveText),
                            KeyCode::Char(value) => Some(EventMessage::InputText(value)),
                            _ => None,
                        }
                    }
                }
            },
            _ => None,
        },
    }
}
