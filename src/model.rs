use std::io;
use ratatui::Frame;

#[derive(PartialEq, Debug)]
pub enum ModelResponse {
    /// Check for another update from the screen model
    NoOp,
    /// Run the ui function on the screen model
    Refresh,
    /// Exit the application
    Exit,
    /// Navigate to a different screen
    NavToMainMenu,
    NavToStrategyCalculator,
    NavToAboutUs,
}

// Note:
// The general idea of this application is simple... it's a loop. That loop
// only knows about one variable... the model. It asks the model to update itself,
// then it asks the model to mutate the given frame.
//
// This allows different screens to be developed in isolation, then quickly added
// to the main application flow when ready.
pub trait Model {
    /// Called by main program loop to update internal state
    fn update(&mut self) -> io::Result<ModelResponse>;

    /// Called by main program loop to refresh/redraw the current screen
    fn ui(&mut self, frame: &mut Frame);
}
