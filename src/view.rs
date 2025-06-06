use std::collections::BTreeMap;

use crate::model::{InputField, Model, Page};

use epub::doc::EpubDoc;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, TableState},
};
use tui_widget_list::{ListBuilder, ListState, ListView};

/// Unit struct for holding the drawing methods of the app
pub struct View;

impl View {
    /// Main draw method from which all other methods are called
    pub fn draw(model: &mut Model, frame: &mut Frame) -> color_eyre::Result<()> {
        frame.render_widget(
            Block::new().style(Style::default().bg(Color::Rgb(20, 20, 20))),
            frame.area(),
        );
        
        let bar_length = 2;
        // Split the TUI into three rows
        // (title bar, main content, and status bar)
        let chunks = Layout::vertical([
            Constraint::Max(bar_length),
            Constraint::Fill(1),
            Constraint::Max(bar_length),
        ])
        .split(frame.area());

        View::draw_title_bar(frame, chunks[0]);
        View::draw_status_bar(model, frame, chunks[2]);

        match Page::VALUES[model.current_page] {
            Page::Home => View::draw_home(frame, chunks[1]),
            Page::SeriesData => View::draw_series_page(model, frame, chunks[1]),
            Page::FileSelection => View::draw_file_selection(model, frame, chunks[1])?,
            Page::BookData => View::draw_book_data_input(model, frame, chunks[1]),
            Page::Loading => View::draw_loading(model, frame, chunks[1])?,
            _ => {}
        };


        Ok(())
    }

    /// Draw the app's title bar
    fn draw_title_bar(frame: &mut Frame, area: Rect) {
        let title_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick);
        let title = Paragraph::new(Text::styled(
            "eBookMetaEditor",
            Style::default().fg(Color::Green),
        ))
        .centered();

        frame.render_widget(title_block, area);
        frame.render_widget(title, View::centered_rect(50, 100, area));
    }

    /// Draw the app's status bar showing the information related to the current page
    fn draw_status_bar(model: &Model, frame: &mut Frame, area: Rect) {
        let status_block = &Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Thick);

        let current_page_string = match Page::VALUES[model.current_page] {
            Page::Home => "Home",
            Page::SeriesData => "Num Series Selection",
            Page::FileSelection => "File Selection",
            Page::BookData => "Book Data Input",
            Page::Loading => "Metadata Edit Loading",
            _ => "",
        };
        let current_page_text = Paragraph::new(Text::styled(
            current_page_string,
            Style::default().fg(Color::Green),
        ))
        .block(
            status_block
                .to_owned()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP),
        )
        .centered();

        let status_chunks = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Fill(1),
            Constraint::Percentage(10),
        ])
        .split(area);
        // frame.render_widget(status_block, area);
        frame.render_widget(
            Paragraph::new(Text::from("<- Ctrl-Left")).centered().block(status_block.to_owned()),
            status_chunks[0],
        );
        frame.render_widget(current_page_text, status_chunks[1]);
        frame.render_widget(
            Paragraph::new(Text::from("Ctrl-Right ->")).centered().block(status_block.to_owned()),
            status_chunks[2],
        );
    }

    /// Draw the app's home page.
    fn draw_home(frame: &mut Frame, area: Rect) {
        let center_block = View::centered_rect(60, 20, area);

        let title_block = Block::default().borders(Borders::ALL);
        let title = Paragraph::new(Text::styled(
            "Press <Ctrl + Left/Right Arrow> to navigate pages.",
            Style::default().fg(Color::Green),
        ))
        .block(title_block)
        .centered();

        frame.render_widget(title, center_block);
    }

    /// Draw the app's number of series selection page
    fn draw_series_page(model: &Model, frame: &mut Frame, area: Rect) {
        let question = Paragraph::new(Text::raw("How many series are you editing?")).centered();
        let num_input = Paragraph::new(Text::raw(model.inputs.series_num.to_string())).centered();

        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(3)])
            .split(View::centered_rect(40, 50, area));
        frame.render_widget(question, chunks[0]);
        frame.render_widget(num_input, chunks[1]);
    }

    /// Draw the app's file selection page
    fn draw_file_selection(
        model: &mut Model,
        frame: &mut Frame,
        area: Rect,
    ) -> color_eyre::Result<()> {
        let current_idx = model.inputs.current_series_num;
        let file_list = &mut model.inputs.file_lists[current_idx];
        let file_builder = ListBuilder::new(|context| {
            let file_name = &file_list.items[context.index];

            let mut style = Style::default().fg(Color::default());
            if context.is_selected {
                if file_name.is_dir() {
                    style = style.bg(Color::Green);
                } else {
                    style = style.bg(Color::Red);
                }
            } else if file_name.is_dir() {
                style = style.fg(Color::Green);
            }

            let text: String;
            if let Some(filename) = file_name.file_name() {
                text = filename.to_string_lossy().to_string();
            } else {
                text = "Unable to read file".to_string();
            }

            let mut block = Block::new();
            if file_list.selected.contains(file_name) {
                block = block
                    .borders(Borders::LEFT)
                    .border_type(BorderType::Thick)
                    .style(Style::default().fg(Color::Red));
            }
            let item = Paragraph::new(Text::styled(text, style)).block(block);

            (item, 1)
        });

        let file_list_widget =
            ListView::new(file_builder, file_list.items.len()).infinite_scrolling(true);
        let state = &mut file_list.state;

        let file_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

        frame.render_stateful_widget(file_list_widget, file_chunks[0], state);
        View::draw_selected_files(model, frame, file_chunks[1])?;

        Ok(())
    }

    /// Draw the sidebar showing selected files
    fn draw_selected_files(model: &Model, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let current_idx = model.inputs.current_series_num;
        let file_list = &model.inputs.file_lists[current_idx];
        let mut files: BTreeMap<String, Vec<Line>> = BTreeMap::new();
        for path in &file_list.selected {
            let dir = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let file = Line::from(path.file_name().unwrap().to_string_lossy());

            files.entry(dir).or_default().push(file);
        }

        let directories: Vec<&String> = files.keys().collect();
        let directories_builder = ListBuilder::new(|context| {
            let directory = directories[context.index];
            let mut directory_block_lines: Vec<Line> =
                Vec::from([Line::from(directory.clone()).style(Style::default().fg(Color::Green))]);
            for file in &files[directory] {
                directory_block_lines.push(file.clone())
            }
            let directory_block = Paragraph::new(Text::from(directory_block_lines));

            (directory_block, files[directory].len() as u16 + 1)
        });

        let directories_list_widget = ListView::new(directories_builder, files.len());

        frame.render_stateful_widget(directories_list_widget, area, &mut ListState::default());

        Ok(())
    }

    /// Draw the page for inputting data for each series
    fn draw_book_data_input(model: &mut Model, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 12),
            Constraint::Ratio(9, 12),
        ])
        .split(area);
        let top_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

        let input_chunks: [Rect; 2] = [
            View::centered_rect(70, 80, top_chunks[0]),
            View::centered_rect(70, 80, top_chunks[1]),
        ];
        let border_color = |field: InputField| {
            if model.inputs.currently_editing == field {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }
        };
        frame.render_widget(
            Block::bordered()
                .border_style(border_color(InputField::Series))
                .title("Series Name"),
            input_chunks[0],
        );
        frame.render_widget(
            Block::bordered()
                .border_style(border_color(InputField::Format))
                .title("Format String"),
            input_chunks[1],
        );

        let inputs: Vec<Paragraph> = Vec::from([
            Paragraph::new(Line::from(
                model.inputs.field_values[model.inputs.current_series_num][&InputField::Series][0]
                    .as_str(),
            )),
            Paragraph::new(Line::from(
                model.inputs.field_values[model.inputs.current_series_num][&InputField::Format][0]
                    .as_str(),
            )),
        ]);

        for i in 0..inputs.len() {
            frame.render_widget(&inputs[i], View::centered_rect(85, 50, input_chunks[i]));
        }
        View::draw_book_order(model, frame, chunks[2]);
    }

    /// Draw the box for showing and giving the order of the books in the series
    fn draw_book_order(model: &mut Model, frame: &mut Frame, area: Rect) {
        let chunk = Layout::default()
            .constraints([Constraint::Min(0)])
            .horizontal_margin(5)
            .split(area)[0];

        let current_series = model.inputs.current_series_num;
        let files = &model.inputs.file_lists[current_series].selected;

        let file_rows: Vec<Row> = (0..files.len())
            .map(|i| {
                let epub = EpubDoc::new(&files[i]).unwrap();
                Row::new(vec![
                    (i + 1).to_string(),
                    model.inputs.field_values[current_series]
                        .get(&InputField::BookTitle)
                        .unwrap()[i]
                        .to_owned(),
                    String::from(&epub.metadata.get("creator").unwrap()[0]),
                    files[i].file_name().unwrap().to_string_lossy().to_string(),
                ])
            })
            .collect();
        // Columns widths are constrained in the same way as Layout...
        let widths = [
            Constraint::Percentage(5),
            Constraint::Percentage(35),
            Constraint::Percentage(10),
            Constraint::Percentage(50),
        ];

        let border_color = |field: InputField| {
            if model.inputs.currently_editing == field {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }
        };

        let files_table = Table::new(file_rows, widths)
            .cell_highlight_style(Style::default().fg(Color::Green))
            .row_highlight_style(Style::default().fg(Color::DarkGray));
        frame.render_widget(
            Block::bordered()
                .title("Book Order")
                .border_style(border_color(InputField::BookOrder)),
            chunk,
        );
        frame.render_stateful_widget(
            Table::new(
                vec![Row::new(vec!["Position", "Title", "Author", "File Path"])],
                widths,
            ),
            View::centered_rect(90, 90, chunk),
            &mut TableState::new(),
        );
        let table_state = &mut model.inputs.file_table_states[current_series];
        frame.render_stateful_widget(files_table, View::centered_rect(90, 80, chunk), table_state);
    }

    fn draw_loading(model: &mut Model, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let selected = &model.all_selected;
        let file_builder = ListBuilder::new(|context| {
            let file_name = &selected[context.index];

            let mut style = Style::default().fg(Color::default());
            style = if model.finished_books.contains(file_name) {
                style.fg(Color::Green)
            } else if context.index == model.current_book {
                style.fg(Color::Red)
            } else {
                style
            };

            let text: String;
            if let Some(filename) = file_name.file_name() {
                text = filename.to_string_lossy().to_string();
            } else {
                text = "Unable to read file".to_string();
            }

            let item = Paragraph::new(Text::styled(text, style));

            (item, 1)
        });

        let file_list_widget = ListView::new(file_builder, selected.len()).infinite_scrolling(true);
        frame.render_stateful_widget(file_list_widget, area, &mut ListState::default());

        if model.current_book < model.all_selected.len() {
            let book = model.all_selected[model.current_book].to_owned();
            model.edit_epub(&book)?;
        }

        Ok(())
    }
}

impl View {
    /// Get a rectangle object centred inside another rect with size (percent_x, percent_y)
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
    }
}
