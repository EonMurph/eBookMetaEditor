use std::path::PathBuf;

enum CurrentScreen {
    Home,
    Quit,
    DataInput,
    BookEdit,
}

pub struct Model {
    pub running: bool,
    current_file_name: Option<PathBuf>,
    file_list: Option<Vec<PathBuf>>,
    series_name: Option<String>,
    book_name: Option<String>,
    format: String,
    current_screen: CurrentScreen,
}

impl Model {
    pub fn new() -> Self {
        Model {
            running: true,
            current_file_name: None,
            file_list: None,
            series_name: None,
            book_name: None,
            format: "{book_name}".into(),
            current_screen: CurrentScreen::Home,
        }
    }
}
