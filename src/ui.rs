use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::constants::TITLE;

pub fn render_border(frame: &mut Frame, screen: Rect) {
    let border_block = Block::default()
        .borders(Borders::all())
        .style(Style::default().fg(Color::White));
    let border = Paragraph::new(Text::default())
        .alignment(Alignment::Center)
        .block(border_block);
    frame.render_widget(border, screen)
}

pub fn render_title_block(frame: &mut Frame, rect: Rect) {
    render_centered_text(frame, rect, TITLE);
}

pub fn render_centered_text(frame: &mut Frame, rect: Rect, text: &str) {
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(paragraph, rect);
}

pub fn render_sub_title_block(frame: &mut Frame, rect: Rect) {
    let sub_title = Paragraph::new("Made by Freeside Software")
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(sub_title, rect);
}

pub trait MenuNavigation {
    fn get_menu_length(&self) -> usize;
    fn get_menu_index(&self) -> i8;
    fn set_menu_index(&mut self, index: i8);

    fn increment_menu_index(&mut self, increment: i8) {
        let current_index = self.get_menu_index();
        if increment < 0 && self.get_menu_index() <= 0 {
            return
        }
        if increment > 0 && self.get_menu_index() >= (self.get_menu_length() - 1) as i8 {
            return
        }
        self.set_menu_index(current_index + increment);

    }
}