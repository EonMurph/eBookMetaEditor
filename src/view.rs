use crate::model::{Model, Page};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

pub struct View;

impl View {
    pub fn draw(model: &mut Model, frame: &mut Frame) {
        match model.current_screen {
            Page::Home => View::draw_home(frame),
            Page::Quit => todo!(),
            _ => todo!(),
        };
    }

    fn draw_home(frame: &mut Frame) {
        let center_block = View::centered_rect(60, 20, frame.area());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(
            "Press <Enter> to start the program.",
            Style::default().fg(Color::Green),
        ))
        .block(title_block)
        .centered();

        frame.render_widget(title, center_block);
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
