use std::{collections::HashSet, fs::canonicalize, path::PathBuf};

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
    /// Page for placing the books in their respective orders
    OrderBooks,
    /// Loading page shown while metadata is being edited
    Loading,
}

impl Page {
    /// List of pages used for cycling through the app pages
    pub const VALUES: [Self; 3] = [Self::Home, Self::SeriesData, Self::FileSelection];
}

/// Struct to hold the data for the list of files
pub struct FileList {
    /// Vector for holding the file paths in the current directory
    pub items: Vec<PathBuf>,
    /// A list state object holding a reference to the current highlighted item
    pub state: ListState,
    /// A hash set of all the files selected
    pub selected: HashSet<PathBuf>,
    /// The path of the current directory
    pub current_directory: PathBuf,
}

impl FromIterator<PathBuf> for FileList {
    /// Create a FileList struct based on an iterable of file paths
    fn from_iter<T: IntoIterator<Item = PathBuf>>(iter: T) -> Self {
        FileList {
            items: iter.into_iter().collect(),
            state: ListState::default(),
            selected: HashSet::default(),
            current_directory: canonicalize(PathBuf::from("./")).unwrap(),
        }
    }
}

/// Struct to hold input field data
pub struct Input {
    /// Integer representing the number of series being edited
    pub series_num: i8,
    /// Integer representing the current series being edited
    pub current_series_num: usize,
    /// Vector holding a FileList struct for each series being edited
    pub file_lists: Vec<FileList>,
}

impl Input {
    /// Initialise an Input struct
    fn new() -> Self {
        Input {
            series_num: 1,
            current_series_num: 0,
            file_lists: Vec::new(),
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
            current_page: 0,
            inputs: Input::new(),
        }
    }
}
