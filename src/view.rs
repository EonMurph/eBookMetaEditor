use crate::model::{Model, Page};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tui_widget_list::{ListBuilder, ListView};

pub struct View;

impl View {
    pub fn draw(model: &mut Model, frame: &mut Frame) -> color_eyre::Result<()> {
        let bar_length = 2;
        // Split the TUI into three rows
        // (title bar, main content, and status bar)
        let chunks = Layout::vertical([
            Constraint::Max(bar_length),
            Constraint::Min(1),
            Constraint::Max(bar_length),
        ])
        .split(frame.area());

        View::draw_title_bar(frame, chunks[0]);
        View::draw_status_bar(frame, chunks[2]);

        match Page::VALUES[model.current_page] {
            Page::Home => View::draw_home(frame, chunks[1]),
            Page::SeriesData => View::draw_series_page(model, frame, chunks[1]),
            Page::FileSelection => View::draw_file_selection(model, frame, chunks[1])?,
            Page::Quit => todo!(),
            _ => todo!(),
        };

        Ok(())
    }

    fn draw_title_bar(frame: &mut Frame, area: Rect) {
        let title_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick);
        let title = Paragraph::new(Text::styled(
            "eBookMetaEditor",
            Style::default().fg(Color::Green),
        ))
        .block(
            Block::default()
                .borders(Borders::RIGHT | Borders::LEFT | Borders::BOTTOM)
                .border_type(BorderType::Rounded)
                .style(Style::default()),
        )
        .centered();

        frame.render_widget(title_block, area);
        frame.render_widget(title, View::centered_rect(50, 100, area));
    }

    fn draw_status_bar(frame: &mut Frame, area: Rect) {
        let status_block = Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Thick);

        frame.render_widget(status_block, area);
    }

    fn draw_home(frame: &mut Frame, area: Rect) {
        let center_block = View::centered_rect(60, 20, area);

        let title_block = Block::default().borders(Borders::ALL);
        let title = Paragraph::new(Text::styled(
            "Press <Alt + Enter> to start the program.",
            Style::default().fg(Color::Green),
        ))
        .block(title_block)
        .centered();

        frame.render_widget(title, center_block);
    }

    fn draw_series_page(model: &Model, frame: &mut Frame, area: Rect) {
        let question = Paragraph::new(Text::raw("How many series are you editing?")).centered();
        let num_input = Paragraph::new(Text::raw(model.inputs.series_num.to_string())).centered();

        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(3)])
            .split(View::centered_rect(40, 50, area));
        frame.render_widget(question, chunks[0]);
        frame.render_widget(num_input, chunks[1]);
    }

    fn draw_file_selection(
        model: &mut Model,
        frame: &mut Frame,
        area: Rect,
    ) -> color_eyre::Result<()> {
        let current_idx = model.inputs.current_series_num;
        let file_list = &mut model.inputs.file_lists[current_idx];
        let builder = ListBuilder::new(|context| {
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

        let list = ListView::new(builder, file_list.items.len()).infinite_scrolling(true);
        let state = &mut file_list.state;
        frame.render_stateful_widget(list, area, state);

        Ok(())
    }

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
