mod app;
mod menu;
mod ui;
mod model;
mod about;
mod constants;

use crate::app::App;
use color_eyre::Result;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{Terminal};
use std::error::Error;
use std::io;
use crate::about::about_us_screen::AboutUsScreen;
use crate::menu::menu_screen::MenuScreen;
use crate::model::{Model, ModelResponse};

fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;

    // Setup Terminal
    let mut terminal = setup_terminal()?;

    // Create App and Run
    let mut app = App::new();
    let app_result = run_app(&mut terminal, &mut app);

    // Restore Terminal
    restore_terminal(&mut terminal)?;

    if let Err(err) = app_result {
        println!("{err:?}")
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, _app: &mut App) -> io::Result<()> {
    let mut screen: Box<dyn Model> = Box::new(MenuScreen::new());
    loop {
        terminal.draw(|f| screen.ui(f))?;

        loop {
            // Nested loop prevents rerender of UI when not necessary
            let response = screen.update();
            match response {
                Ok(ModelResponse::Refresh) => break, // break into parent look to rerender
                Ok(ModelResponse::Exit) => return Ok(()),
                Ok(ModelResponse::NavToMainMenu) => {
                    screen = Box::new(MenuScreen::new());
                    break;
                }
                Ok(ModelResponse::NavToAboutUs) => {
                    screen = Box::new(AboutUsScreen::new());
                    break;
                }
                _ => break,
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}