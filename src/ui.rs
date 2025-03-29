use std::rc::Rc;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::constants::TITLE;

// Constants for layout dimensions
const HEADER_HEIGHT: u16 = 4;
const FOOTER_HEIGHT: u16 = 4;
const SIDE_MARGIN: u16 = 4;
const SECTION_TITLE_HEIGHT: u16 = 10;

/// Creates the main vertical layout: header, content, footer
pub fn create_common_layout(area: Rect) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(HEADER_HEIGHT),
        Constraint::Min(5),
        Constraint::Length(FOOTER_HEIGHT),
    ]).split(area)
}

pub fn create_header_main_footer_layout(area: Rect,
                                        header_height: u16,
                                        main_height: u16,
                                        footer_height: u16) -> Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(main_height),
        Constraint::Length(footer_height),
    ]).split(area)
}

/// Splits content area into left and right sections (settings and main)
pub fn split_content_horizontally(area: Rect) -> Rc<[Rect]> {
    Layout::horizontal([
        Constraint::Ratio(1, 4),  // Left section (1/4 width)
        Constraint::Ratio(3, 4),  // Right section (3/4 width)
    ])
        .split(area)
}

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

pub fn render_footer_spans(frame: &mut Frame, specific_spans: Vec<String>, rect: Rect) {
    let mut spans = vec![
        " Q ".to_string(), " Quit ".to_string(),
        " M ".to_string(), " Menu ".to_string(),
        " J/↓ ".to_string(), " Down ".to_string(),
        " K/↑ ".to_string(), " Down ".to_string(),
        " H/→ ".to_string(), " Down ".to_string(),
        " L/← ".to_string(), " Down ".to_string()
    ];

    spans.extend(specific_spans);

    let styles = [
        Style::default().bg(Color::Gray).fg(Color::DarkGray),
        Style::default().fg(Color::DarkGray),
    ];

    frame.render_widget(
        Line::from(
            spans
                .iter()
                .enumerate()
                .map(|(idx, content)| Span::styled(content, styles[idx % 2]))
                .collect::<Vec<_>>(),
        ).left_aligned(),
        rect
    );
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