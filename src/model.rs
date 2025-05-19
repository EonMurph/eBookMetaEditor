use std::{collections::HashSet, fs::canonicalize, path::PathBuf};

use tui_widget_list::ListState;

pub enum Page {
    Home,
    Quit,
    SeriesData,
    FileSelection,
    BookData,
    OrderBooks,
    Loading,
}

impl Page {
    pub const VALUES: [Self; 3] = [Self::Home, Self::SeriesData, Self::FileSelection];
}

// Struct to hold the data for the list of files
pub struct FileList {
    pub items: Vec<PathBuf>,
    pub state: ListState,
    pub selected: HashSet<PathBuf>,
    pub current_directory: PathBuf,
}

impl FromIterator<PathBuf> for FileList {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        FileList {
            items: iter.into_iter().collect(),
            state: ListState::default(),
            selected: HashSet::default(),
            current_directory: canonicalize(PathBuf::from("./")).unwrap(),
        }
    }
}

// Struct to hold input field data
pub struct Input {
    pub series_num: i8,
    pub current_series_num: usize,
    pub file_lists: Vec<FileList>,
}

impl Input {
    fn new() -> Self {
        Input {
            series_num: 1,
            current_series_num: 0,
            file_lists: Vec::new(),
        }
    }
}

pub struct Model {
    pub running: bool,
    pub current_page: usize,
    pub inputs: Input,
}

impl Model {
    pub fn new() -> Self {
        Model {
            running: true,
            current_page: 0,
            inputs: Input::new(),
        }
    }
}
