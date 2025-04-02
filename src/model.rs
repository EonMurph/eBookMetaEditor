use std::{collections::HashSet, path::PathBuf};

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

// Struct to hold the data for the list of files
pub struct FileList {
    pub items: Vec<PathBuf>,
    pub state: ListState,
    pub selected: HashSet<usize>,
    pub current_directory: PathBuf,
}

impl FromIterator<PathBuf> for FileList {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        FileList {
            items: iter.into_iter().collect(),
            state: ListState::default(),
            selected: HashSet::default(),
            current_directory: PathBuf::from("./"),
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
    current_file_name: Option<PathBuf>,
    files_list: Option<Vec<FileList>>,
    num_series: i8,
    series_name: Option<String>,
    book_name: Option<String>,
    format: String,
    pub current_page: Page,
    pub inputs: Input,
}

impl Model {
    pub fn new() -> Self {
        Model {
            running: true,
            current_file_name: None,
            files_list: None,
            num_series: 0,
            series_name: None,
            book_name: None,
            format: "{book_name}".into(),
            current_page: Page::Home,
            inputs: Input::new(),
        }
    }

    pub fn set_num_series(&mut self) {
        self.num_series = self.inputs.series_num;
    }
}
