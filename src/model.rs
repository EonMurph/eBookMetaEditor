use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs::{File, canonicalize, read, read_dir},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use ratatui::widgets::TableState;
use regex::Regex;
use tempfile::{TempDir, tempdir};
use tui_widget_list::ListState;
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

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
    pub const VALUES: [Self; 5] = [
        Self::Home,
        Self::SeriesData,
        Self::FileSelection,
        Self::BookData,
        Self::Loading,
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
    Series,
    Format,
    BookOrder,
    BookTitle,
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
    pub field_values: Vec<HashMap<InputField, Vec<String>>>,
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
            currently_editing: InputField::Series,
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
    /// Vec containing all the selected books
    pub all_selected: Vec<PathBuf>,
    /// Vec containing a HashMap of all the inputs
    pub all_field_values: Vec<HashMap<InputField, String>>,
    /// Hashset of the editied books
    pub finished_books: HashSet<PathBuf>,
    /// Integer representing the index of the current book being edited
    pub current_book: usize,
}

impl Model {
    /// Initialise a Model struct
    pub fn new() -> Self {
        Model {
            running: true,
            current_page: 3,
            inputs: Input::new(),
            all_selected: Vec::new(),
            all_field_values: Vec::new(),
            finished_books: HashSet::new(),
            current_book: 0,
        }
    }

    /// Generate the files for the current directory
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

    /// Given a reference to a book's path edit the metadata based on the inputs given
    pub fn edit_epub(&mut self, epub_path: &PathBuf) -> color_eyre::Result<()> {
        let temp_dir = self.prep_epub(epub_path)?;
        let (mut meta_file, mut metadata) = self.get_metadata(temp_dir.path())?;
        metadata = self.edit_metadata(metadata)?;
        meta_file.write_all(metadata.as_bytes())?;
        self.repackage_epub(temp_dir.path(), PathBuf::from(epub_path))?;

        temp_dir.close()?;

        self.finished_books.insert(epub_path.to_owned());
        self.current_book += 1;
        self.current_book = self.current_book.clamp(0, self.all_selected.len() - 1);

        Ok(())
    }

    /// Generate the temperary zip file for the epub's files
    fn prep_epub(&self, epub_path: &PathBuf) -> color_eyre::Result<TempDir> {
        let temp_dir = tempdir()?;

        let file = File::open(epub_path)?;
        let mut archive = ZipArchive::new(&file)?;
        archive.extract(&temp_dir)?;

        Ok(temp_dir)
    }

    /// Get the epub's content.opf and metadata
    fn get_metadata(&self, temp_dir: &Path) -> color_eyre::Result<(File, String)> {
        let extracted_files = WalkDir::new(temp_dir)
            .into_iter()
            .filter_map(|entry| entry.ok());
        for extracted_file in extracted_files {
            if extracted_file.path().file_name().unwrap_or_default() == "content.opf" {
                let mut content = String::new();
                File::open(extracted_file.path())
                    .unwrap()
                    .read_to_string(&mut content)?;
                return Ok((File::create(extracted_file.into_path())?, content));
            }
        }

        Err(color_eyre::eyre::eyre!("content.opf not found"))
    }

    /// Edit the metadata based on the inputs given
    fn edit_metadata(&self, mut metadata: String) -> color_eyre::Result<String> {
        let current_book_inputs = &self.all_field_values[self.current_book];
        if let Some(format_string) = current_book_inputs.get(&InputField::Format) {
            let format_string = format_string.as_str();
            let position = &format!(
                "{:0>2}",
                (&current_book_inputs[&InputField::BookOrder].parse::<u32>()? + 1).to_string()
            );
            let title = &current_book_inputs[&InputField::BookTitle];
            let series = &current_book_inputs[&InputField::Series];
            let title_re = Regex::new(r#"(<.*(title|meta).*>)(.+)(</.*(title|meta).*>)"#)?;
            let sort_re = Regex::new(r#"(title_sort.*content=")(.*)("/>)"#)?;
            let substitutions =
                HashMap::from([("position", position), ("title", title), ("series", series)]);
            let formatted_string = subst::substitute(format_string, &substitutions)?;

            metadata = title_re
                .replace_all(&metadata, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], formatted_string, &caps[4])
                })
                .to_string();

            metadata = sort_re
                .replace_all(&metadata, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], formatted_string, &caps[3])
                })
                .to_string();
        }

        Ok(metadata)
    }

    /// Repackage the epub's files into an epub with the new metadata
    fn repackage_epub(&self, temp_dir: &Path, output_path: PathBuf) -> color_eyre::Result<()> {
        let temp_file = File::create(output_path)?;
        let mut zip = ZipWriter::new(temp_file);

        zip.start_file("mimetype", SimpleFileOptions::default())?;
        zip.write_all("application/epub+zip".as_bytes())?;

        for file in WalkDir::new(temp_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let file = file.path();
            if let Some(file_name) = file.file_name() {
                if file_name == OsStr::new("mimetype") {
                    continue;
                }
            }

            if file.is_file() {
                let relative_path = file.strip_prefix(temp_dir)?.to_string_lossy();
                let options =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

                zip.start_file(relative_path, options)?;
                let contents = read(file)?;
                zip.write_all(&contents)?;
            }
        }

        zip.finish()?;

        Ok(())
    }
}
