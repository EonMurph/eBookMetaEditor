use std::{
    collections::HashMap,
    fs::{canonicalize, read_dir},
    path::PathBuf,
};

use ratatui::widgets::TableState;
use tui_widget_list::ListState;

/// Enum of pages used in the app
pub enum Page {
    /// Home page
    Home,
    /// Quit page
    Quit,
    /// Page for selecting the number of series to edit
    SeriesData,
    /// Page for selecting the files needed to be edited for that series
    FileSelection,
    /// Page for inputting the data for each book in the series
    BookData,
    /// Loading page shown while metadata is being edited
    Loading,
}

impl Page {
    /// List of pages used for cycling through the app pages
    pub const VALUES: [Self; 4] = [
        Self::Home,
        Self::SeriesData,
        Self::FileSelection,
        Self::BookData,
    ];
}

/// Struct to hold the data for the list of files
pub struct FileList {
    /// Vector for holding the file paths in the current directory
    pub items: Vec<PathBuf>,
    /// A list state object holding a reference to the current highlighted item
    pub state: ListState,
    /// A vector of all the files selected
    pub selected: Vec<PathBuf>,
    /// The path of the current directory
    pub current_directory: PathBuf,
}

impl FromIterator<PathBuf> for FileList {
    /// Create a FileList struct based on an iterable of file paths
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        FileList {
            items: iter.into_iter().collect(),
            state: ListState::default(),
            selected: Vec::new(),
            current_directory: canonicalize(PathBuf::from("./")).unwrap(),
        }
    }
}

/// Struct to hold possible input fields for editing
#[derive(Hash, Eq, PartialEq)]
pub enum InputField {
    Author,
    Series,
    Format,
    BookOrder,
}

/// Struct to hold input field data
pub struct Input {
    /// Integer representing the number of series being edited
    pub series_num: i8,
    /// Integer representing the current series being edited
    pub current_series_num: usize,
    /// Vector holding a FileList struct for each series being edited
    pub file_lists: Vec<FileList>,
    /// InputField representing the current field being edited
    pub currently_editing: InputField,
    /// Vector of HashMaps with key of InputField and value of the String that field is holding
    pub field_values: Vec<HashMap<InputField, String>>,
    /// State of the Table of selected book
    pub file_table_states: Vec<TableState>,
}

impl Input {
    /// Initialise an Input struct
    fn new() -> Self {
        Input {
            series_num: 1,
            current_series_num: 0,
            file_lists: Vec::new(),
            currently_editing: InputField::Author,
            field_values: Vec::new(),
            file_table_states: Vec::new(),
        }
    }
}

/// Struct holding the data for the app
pub struct Model {
    /// Boolean representing whether the app is running or not
    pub running: bool,
    /// Integer representing the current page
    pub current_page: usize,
    /// The Input struct for the app
    pub inputs: Input,
}

impl Model {
    /// Initialise a Model struct
    pub fn new() -> Self {
        Model {
            running: true,
            current_page: 3,
            inputs: Input::new(),
        }
    }

    pub fn get_current_file_list(&self, directory: PathBuf) -> Vec<PathBuf> {
        let directory_contents: Vec<PathBuf> = read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| canonicalize(entry.path()).unwrap())
            .collect();
        let mut files_list: Vec<PathBuf> = Vec::new();
        {
            let mut directories: Vec<PathBuf> = Vec::new();
            let mut files: Vec<PathBuf> = Vec::new();
            for entry in directory_contents {
                if entry
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .chars()
                    .nth(0)
                    .unwrap()
                    != '.'
                {
                    if entry.is_dir() {
                        directories.push(entry);
                    } else if entry.is_file() {
                        if let Some(extension) = entry.extension() {
                            if extension == "epub" {
                                files.push(entry);
                            }
                        }
                    }
                }
            }
            directories.sort();
            files.sort();
            files_list.append(&mut directories);
            files_list.append(&mut files);
        }

        files_list
    }
}
