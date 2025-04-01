use std::path::PathBuf;

pub enum Page {
    Home,
    Quit,
    SeriesData,
    BookData,
    OrderBooks,
    Loading,
}


// Struct to hold input field data
pub struct Input {
    pub series_num: i8,
}

impl Input {
    fn new() -> Self {
        Input {
            series_num: 0,
        }
    }
}

pub struct Model {
    pub running: bool,
    current_file_name: Option<PathBuf>,
    file_list: Option<Vec<PathBuf>>,
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
            file_list: None,
            num_series: 0,
            series_name: None,
            book_name: None,
            format: "{book_name}".into(),
            current_page: Page::SeriesData,
            inputs: Input::new(),
        }
    }
        
    pub fn set_num_series(&mut self) {
        self.num_series = self.inputs.series_num;
    }
}
