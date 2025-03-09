/// `App` stores the application state for the TUI.
///
/// While currently empty, this struct will be expanded to contain various
/// pieces of state as the application grows, such as:
/// - User input and selections
/// - Data being displayed
/// - UI navigation state
/// - Application configuration
pub struct App {}

impl App {
    /// Creates a new instance of the application state.
    ///
    /// Returns an empty `App` struct, which will be populated with state
    /// as features are added to the application.
    ///
    /// # Examples
    ///
    /// ```
    /// let app = App::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }
}