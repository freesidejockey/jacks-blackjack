use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use crate::constants::{ABOUT_US, ABOUT_US_TEXT};
use crate::model::{Model, ModelResponse};
use crate::ui::{render_border, render_centered_text, render_sub_title_block, render_title_block};

// ---- About Us Screen ----
pub struct AboutUsScreen {}
impl AboutUsScreen {
    pub fn new() -> Self {
        Self {}
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
                _ => Ok(ModelResponse::Refresh),
            }
        }
        Ok(ModelResponse::Refresh)
    }

    fn ui(&mut self, frame: &mut Frame) {
        // We will use the entire screen
        let screen = frame.area();
        render_border(frame, screen);

        // break the screen into chunks
        let menu_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(14),
                Constraint::Min(10),
            ])
            .split(screen);

        render_centered_text(frame, menu_layout[0], ABOUT_US);
        render_centered_text(frame, menu_layout[1], ABOUT_US_TEXT);
    }
}