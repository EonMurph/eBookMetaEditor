use crate::model::{Model, Page};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub struct View;

impl View {
    pub fn draw(model: &Model, frame: &mut Frame) {
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

        match model.current_page {
            Page::Home => View::draw_home(frame, chunks[1]),
            Page::SeriesData => View::draw_series_page(model, frame, chunks[1]),
            Page::Quit => todo!(),
            _ => todo!(),
        };
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

        let title_block = Block::default()
            .borders(Borders::ALL);
        let title = Paragraph::new(Text::styled(
            "Press <Enter> to start the program.",
            Style::default().fg(Color::Green),
        ))
        .block(title_block)
        .centered();

        frame.render_widget(title, center_block);
    }

    fn draw_series_page(model: &Model, frame: &mut Frame, area: Rect) {
        let question = Paragraph::new(Text::raw(
            "How many series are you editing?",
        )).centered();
        let num_input = Paragraph::new(Text::raw(
            model.inputs.series_num.to_string()
        )).centered();

        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(3)]).split(
            View::centered_rect(40, 50, area)
        );
        frame.render_widget(question, chunks[0]);
        frame.render_widget(num_input, chunks[1]);
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
