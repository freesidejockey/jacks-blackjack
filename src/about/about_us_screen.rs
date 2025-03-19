use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use crate::constants::{ABOUT_US, ABOUT_US_TEXT};
use crate::model::{Model, ModelResponse};
use crate::ui::{render_border, render_centered_text, render_sub_title_block, render_title_block};

// ---- About Us Screen ----
pub struct AboutUsScreen {
    scroll_offset: usize,
    scroll_necessary: bool
}

impl AboutUsScreen {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            scroll_necessary: false
        }
    }
}

// ---- TRAIT IMPLEMENTATIONS ----
impl Model for AboutUsScreen {
    fn update(&mut self) -> std::io::Result<ModelResponse> {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                return Ok(ModelResponse::Refresh);
            }
            return match key.code {
                KeyCode::Char('q') => Ok(ModelResponse::Exit),
                KeyCode::Char('m') => Ok(ModelResponse::NavToMainMenu),
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.scroll_offset > 0 {
                        if self.scroll_offset == 2 {
                            self.scroll_offset -= 2;
                        } else {
                            self.scroll_offset -= 1;
                        }
                    }
                    Ok(ModelResponse::Refresh)
                },
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.scroll_necessary {
                        if self.scroll_offset == 0 {
                            self.scroll_offset += 2;
                        } else {
                            self.scroll_offset += 1;
                        }
                    }
                    Ok(ModelResponse::Refresh)
                },
                _ => Ok(ModelResponse::Refresh),
            }
        }
        Ok(ModelResponse::Refresh)
    }

    fn ui(&mut self, frame: &mut Frame) {
        let screen = frame.area();
        render_border(frame, screen);

        // break the screen into chunks
        let menu_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14),
                Constraint::Min(10),
                Constraint::Length(4),
            ])
            .split(screen);

        render_centered_text(frame, menu_layout[0], ABOUT_US);

        // Split ABOUT_US_TEXT into lines
        let text_lines: Vec<&str> = ABOUT_US_TEXT.split('\n').collect();

        // Calculate the visible area height (accounting for indicator rows)
        let content_area = menu_layout[1];
        let needs_top_indicator = self.scroll_offset > 0;
        let max_scroll = text_lines.len().saturating_sub(content_area.height as usize - 2);
        let needs_bottom_indicator = self.scroll_offset < max_scroll;

        // Adjust scroll position if needed
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
        if text_lines.len() >= content_area.height as usize {
            self.scroll_necessary = true;
        }

        // Create layout for indicators and content
        let indicator_constraints = match (needs_top_indicator, needs_bottom_indicator) {
            (true, true) => vec![
                Constraint::Length(1),  // Top indicator
                Constraint::Min(1),     // Content
                Constraint::Length(1),  // Bottom indicator
            ],
            (true, false) => vec![
                Constraint::Length(1),  // Top indicator
                Constraint::Min(1),     // Content
            ],
            (false, true) => vec![
                Constraint::Min(1),     // Content
                Constraint::Length(1),  // Bottom indicator
            ],
            (false, false) => vec![
                Constraint::Min(1),     // Content only
            ],
        };

        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(indicator_constraints)
            .split(content_area);

        // Calculate visible content area and height
        let text_area_idx = if needs_top_indicator { 1 } else { 0 };
        let text_area = content_chunks[text_area_idx];
        let visible_height = text_area.height as usize;

        // Create a slice of visible lines based on scroll position
        let end_idx = (self.scroll_offset + visible_height).min(text_lines.len());
        let visible_text = text_lines[self.scroll_offset..end_idx].join("\n");

        // Render content
        render_centered_text(frame, text_area, &visible_text);

        // Add scroll indicators
        use ratatui::widgets::{Paragraph};
        use ratatui::style::{Style, Color};
        use ratatui::layout::Alignment;

        if needs_top_indicator {
            let up_indicator = Paragraph::new("↑")
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            frame.render_widget(up_indicator, content_chunks[0]);
        }

        if needs_bottom_indicator {
            let bottom_idx = content_chunks.len() - 1;
            let down_indicator = Paragraph::new("↓")
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            frame.render_widget(down_indicator, content_chunks[bottom_idx]);
        }
    }
}