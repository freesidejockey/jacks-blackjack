use std::collections::HashMap;
use std::fs;
use crate::model::{Model, ModelResponse};
use crate::ui::{render_border, render_centered_text, MenuNavigation};
use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use crate::logic::strategy_calculator_logic::{BlackjackStrategy, SurrenderRule};

// ---- Adjustable Settings ----
enum AdjustableOption {
    NumberOfDecks,
    Soft17DealerAction,
    AllowDoubleAfterSplit,
    AllowSurrender,
    DealerPeak,
}

impl AdjustableOption {
    pub fn to_string(&self) -> String {
        match self {
            AdjustableOption::NumberOfDecks => "Number of Decks".to_string(),
            AdjustableOption::Soft17DealerAction => "Soft 17 Dealer Action".to_string(),
            AdjustableOption::AllowDoubleAfterSplit => "Allow Double After Split".to_string(),
            AdjustableOption::AllowSurrender => "Allow Surrender".to_string(),
            AdjustableOption::DealerPeak => "Dealer Peak".to_string(),
        }
    }
}

const ADJUSTABLE_OPTIONS: [AdjustableOption; 5] = [
    AdjustableOption::NumberOfDecks,
    AdjustableOption::Soft17DealerAction,
    AdjustableOption::AllowDoubleAfterSplit,
    AdjustableOption::AllowSurrender,
    AdjustableOption::DealerPeak,
];

// ---- Strategy Calculator Screen ----
pub struct StrategyCalculatorScreen {
    active_menu_index: i8,
    number_of_decks: i8,
    dealer_stands_on_soft_17: bool,
    allow_double_after_split: bool,
    surrender_rule: SurrenderRule,
    dealer_peak: bool,
    strategy: BlackjackStrategy,
    strategy_cache: HashMap<String, BlackjackStrategy>,
    active_strategy_name: String,
}

impl StrategyCalculatorScreen {
    pub fn new() -> Self {
        // Initialize with default game settings
        let default_decks = 1;
        let default_dealer_stands_on_soft_17 = true;
        let default_double_after_split = true;
        let default_surrender = SurrenderRule::NotAllowed;
        let default_dealer_peak = true;

        // Load all strategies from the strategies directory
        let strategies_dir = "resources/strategies";
        let mut strategy_cache = HashMap::new();

        // Default strategy to load if we can't find any
        let mut default_strategy = BlackjackStrategy::new();
        let mut active_strategy_name = "Default".to_string();

        // Attempt to read directory and load all .json files
        if let Ok(entries) = fs::read_dir(strategies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if let Some(filename) = path.file_stem().and_then(|name| name.to_str()) {
                        match BlackjackStrategy::from_file(path.to_str().unwrap()) {
                            Ok(strategy) => {
                                // Cache the strategy
                                strategy_cache.insert(filename.to_string(), strategy);
                            },
                            Err(e) => {
                                // Just print error and continue - don't want to crash the app
                                eprintln!("Error loading strategy {}: {}", filename, e);
                            }
                        }
                    }
                }
            }
        }

        // Find a matching strategy using our static method
        if let Some((name, strategy)) = Self::find_matching_strategy(
            &strategy_cache,
            default_decks,
            default_dealer_stands_on_soft_17,  // Note the inversion for dealer_stands_on_soft_17
            default_double_after_split,
            default_dealer_peak,
            default_surrender
        ) {
            active_strategy_name = name;
            default_strategy = strategy.clone();  // Assuming your strategy type implements Clone
        }

        Self {
            active_menu_index: 0,
            number_of_decks: default_decks,
            dealer_stands_on_soft_17: default_dealer_stands_on_soft_17,
            allow_double_after_split: default_double_after_split,
            surrender_rule: default_surrender,
            dealer_peak: default_dealer_peak,
            strategy: default_strategy,
            strategy_cache,
            active_strategy_name,
        }
    }

    // Static method that doesn't require &self
    pub fn find_matching_strategy(
        strategy_cache: &HashMap<String, BlackjackStrategy>,
        decks: i8,
        dealer_stands_on_soft_17: bool,
        double_after_split: bool,
        dealer_peak: bool,
        surrender_rule: SurrenderRule
    ) -> Option<(String, &BlackjackStrategy)> {
        // Iterate through all cached strategies
        for (name, strategy) in strategy_cache {
            // Check for exact match on all variables
            if strategy.rules.decks == decks as u8 &&
                strategy.rules.dealer_stands_on_soft_17 == dealer_stands_on_soft_17 &&
                strategy.rules.double_after_split == double_after_split &&
                strategy.rules.dealer_peak == dealer_peak &&
                strategy.rules.surrender_allowed == surrender_rule {

                // Return the name and reference to the matching strategy
                return Some((name.clone(), strategy));
            }
        }

        // No exact match found
        None
    }

    pub fn find_exact_matching_strategy(&self,
                                        decks: u8,
                                        dealer_stands_on_soft_17: bool,
                                        double_after_split: bool,
                                        dealer_peak: bool,
                                        surrender_rule: SurrenderRule) -> Option<String> {

        // Iterate through all cached strategies
        for (name, strategy) in &self.strategy_cache {
            // Check for exact match on all variables
            if strategy.rules.decks == decks &&
                strategy.rules.dealer_stands_on_soft_17 == dealer_stands_on_soft_17 &&
                strategy.rules.double_after_split == double_after_split &&
                strategy.rules.dealer_peak == dealer_peak &&
                strategy.rules.surrender_allowed == surrender_rule {

                // Return the name of the matching strategy
                return Some(name.clone());
            }
        }

        // No exact match found
        None
    }

    // Add a method to switch active strategy
    pub fn switch_strategy(&mut self, strategy_name: &str) -> bool {
        if let Some(strategy) = self.strategy_cache.get(strategy_name) {
            self.strategy = strategy.clone();
            self.active_strategy_name = strategy_name.to_string();
            true
        } else {
            false
        }
    }

    // Add a method to get strategy names
    pub fn get_strategy_names(&self) -> Vec<String> {
        self.strategy_cache.keys().cloned().collect()
    }

    pub fn create_strategy_key(decks: u8,
                               dealer_stands_on_soft_17: bool,
                               double_after_split: bool,
                               dealer_peak: bool,
                               surrender_allowed: SurrenderRule) -> String {

        // Format: "decks{d}_s17{s}_das{d}_peak{p}_surr{s}"
        format!(
            "decks{}_s17{}_das{}_peak{}_surr{}",
            decks,
            if dealer_stands_on_soft_17 { "y" } else { "n" },
            if double_after_split { "y" } else { "n" },
            if dealer_peak { "y" } else { "n" },
            surrender_allowed.to_string(),
        )
    }

    pub fn update_strategy_based_on_settings(&mut self) {
        // Convert the UI settings to strategy variables
        let decks = self.number_of_decks as u8;
        let dealer_stands_on_soft_17 = self.dealer_stands_on_soft_17;
        let double_after_split = self.allow_double_after_split;
        let dealer_peak = self.dealer_peak;
        let surrender_allowed =self.surrender_rule;

        // Find an exact matching strategy
        if let Some(matching_strategy) = self.find_exact_matching_strategy(
            decks,
            dealer_stands_on_soft_17,
            double_after_split,
            dealer_peak,
            surrender_allowed
        ) {
            // Update the active strategy if we found a match
            self.switch_strategy(&matching_strategy);
        } else {
            // No exact match found - use a default or notify the user
            // For now, let's use "random-strategy" as a fallback
            if self.strategy_cache.contains_key("random-strategy") {
                self.switch_strategy("random-strategy");
            }

            // You might want to log this missing combination for future strategy creation:
            let key = Self::create_strategy_key(
                decks,
                dealer_stands_on_soft_17,
                double_after_split,
                dealer_peak,
                surrender_allowed
            );

            // Disabling logging as it logs into the UI
            // eprintln!("No strategy found for combination: {}", key);
        }
    }

    fn render_menu_body(&self, frame: &mut Frame, rect: Rect) {
        let mut menu_body: Vec<Line<'_>> = vec![];

        for (i, item) in ADJUSTABLE_OPTIONS.iter().enumerate() {
            menu_body.push(Line::from(""));
            let mut text = if self.active_menu_index == i as i8 {
                "> ".to_string()
            } else {
                String::new()
            };

            text.push_str(item.to_string().as_str());

            match i {
                0 => text.push_str(&format!(": < {} >", self.number_of_decks)),
                1 => text.push_str(&format!(": < {} >", if self.dealer_stands_on_soft_17 { "Dealer Stands" } else { "Dealer Hits" })),
                2 => text.push_str(&format!(": < {} >", if self.allow_double_after_split { "Allowed" } else { "Not Allowed" })),
                3 => text.push_str(&format!(": < {} >", self.surrender_rule.to_string())),
                4 => text.push_str(&format!(": < {} >", if self.dealer_peak { "Yes" } else { "No" })),
                _ => {}, // Handle any other case
            }

            if self.active_menu_index == i as i8 {
                menu_body.push(Line::from(text).fg(Color::Green))
            } else {
                menu_body.push(Line::from(text));
            }

        }

        let menu_options = Paragraph::new(menu_body)
            .bold()
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(menu_options, rect);
    }

    fn increment_current_menu_item(&mut self, increment: i8) {
        let mut hm_one = HashMap::new();
        hm_one.insert(1, SurrenderRule::NotAllowed);
        hm_one.insert(2, SurrenderRule::AnyUpcard);
        hm_one.insert(3, SurrenderRule::Dealer2Through10);

        let mut hm_two = HashMap::new();
        hm_two.insert(SurrenderRule::NotAllowed, 1);
        hm_two.insert(SurrenderRule::AnyUpcard, 2);
        hm_two.insert(SurrenderRule::Dealer2Through10, 3);

        let menu_item = ADJUSTABLE_OPTIONS.get(self.active_menu_index as usize).unwrap();
        match menu_item {
            AdjustableOption::NumberOfDecks => {
                if increment < 0 && self.number_of_decks <= 1 {
                    return;
                }
                if increment > 0 && self.number_of_decks >= 6 {
                    return;
                }
                self.number_of_decks += increment;
            }
            AdjustableOption::Soft17DealerAction => {
                self.dealer_stands_on_soft_17 = !self.dealer_stands_on_soft_17;
            }
            AdjustableOption::AllowDoubleAfterSplit => {
                self.allow_double_after_split = !self.allow_double_after_split;
            }
            AdjustableOption::AllowSurrender => {
                let curr_idx = hm_two.get(&self.surrender_rule).unwrap();
                if *curr_idx == 1i8 && increment < 0 {
                    self.surrender_rule = hm_one.get(&(3)).unwrap().clone();
                } else if *curr_idx == 3i8 && increment > 0 {
                    self.surrender_rule = hm_one.get(&(1)).unwrap().clone();
                } else {
                    self.surrender_rule = hm_one.get(&(curr_idx + increment)).unwrap().clone();
                }
            }
            AdjustableOption::DealerPeak => {
                self.dealer_peak = !self.dealer_peak;
            }
        }
    }

    pub fn render_table(&mut self, frame: &mut Frame, rect: Rect, title: &str) {
        // First, create an inner area with some margins to center the table
        let inner_rect = rect.inner(Margin {
            vertical: 0,                               // Keep vertical space the same
            horizontal: rect.width.saturating_sub(26) / 2  // Center horizontally (table width is ~26)
        });

        let rows = [
            Row::new(vec!["5", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["6", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["7", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["8", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["9", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["10", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["11", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["12", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["13", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["14", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["15", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["16", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["17", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["18", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["19", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["20", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
            Row::new(vec!["21", "H", "H", "H", "H", "H", "H", "H", "H", "H", "H"]),
        ];
        let widths = [
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
        ];

        let header_cells = vec![" ", "2", "3", "4", "5", "6", "7", "8", "9", "10", "A"]
            .into_iter()
            .map(|h| {
                Cell::new(h)
                    .style(Style::new().bold())
            })
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::new().bold())
            .bottom_margin(1)
            .top_margin(1);

        let table = Table::new(rows, widths)
            // .column_spacing(1)
            .style(Style::new().blue())
            .header(header)
            .block(Block::new().title(title).style(Style::new().bold()).title_alignment(Alignment::Center))
            .row_highlight_style(Style::new().reversed())
            .column_highlight_style(Style::new().red())
            .cell_highlight_style(Style::new().blue())
            .highlight_symbol(">>");

        frame.render_widget(table, inner_rect);
    }

    pub fn render_hard_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from hard hands data in the strategy
        let rows = self.strategy.tables.hard_hands.iter().map(|row| {
            // Convert total to string for the first column
            let total_str = row.total.to_string();

            // Create a vector with the total as the first element
            let mut row_cells = vec![total_str];

            // Add all action codes from the row
            row_cells.extend(row.actions.iter().cloned());

            // Create a Row with all cells
            Row::new(row_cells)
        }).collect::<Vec<_>>();

        // Define column widths - one for the hand total, and one for each dealer card
        let mut widths = vec![Constraint::Length(2)]; // Width for hand total column
        widths.extend(vec![Constraint::Length(2); 10]); // Width for dealer card columns (2-A)

        // Create header cells
        let header_cells = vec![" ", "2", "3", "4", "5", "6", "7", "8", "9", "10", "A"]
            .into_iter()
            .map(|h| Cell::new(h).style(Style::new().bold()))
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::new().bold())
            .bottom_margin(1)
            .top_margin(1);

        // Create inner area with margins to center the table
        let inner_rect = rect.inner(Margin {
            vertical: 0,
            horizontal: rect.width.saturating_sub(26) / 2
        });

        let table = Table::new(rows, widths)
            .style(Style::new().blue())
            .header(header)
            .block(Block::new()
                .title("Hard Hands")
                .style(Style::new().bold())
                .title_alignment(Alignment::Center))
            .row_highlight_style(Style::new().reversed())
            .column_highlight_style(Style::new().red())
            .cell_highlight_style(Style::new().blue())
            .highlight_symbol(">>");

        frame.render_widget(table, inner_rect);
    }

    pub fn render_soft_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from soft hands data in the strategy
        let rows = self.strategy.tables.soft_hands.iter().map(|row| {
            // For soft hands, we want to show "A+X" format instead of just the total
            // The soft hand total is always Ace (11) + some value, so we can extract that value
            let second_card = row.total - 11;
            let hand_display = format!("A{}", second_card);

            // Create a vector with the formatted hand as the first element
            let mut row_cells = vec![hand_display];

            // Add all action codes from the row
            row_cells.extend(row.actions.iter().cloned());

            // Create a Row with all cells
            Row::new(row_cells)
        }).collect::<Vec<_>>();

        // Rest of the method remains the same...
        let mut widths = vec![Constraint::Length(3)]; // Increased width for "A+X" format
        widths.extend(vec![Constraint::Length(2); 10]);

        let header_cells = vec![" ", "2", "3", "4", "5", "6", "7", "8", "9", "10", "A"]
            .into_iter()
            .map(|h| Cell::new(h).style(Style::new().bold()))
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::new().bold())
            .bottom_margin(1)
            .top_margin(1);

        let inner_rect = rect.inner(Margin {
            vertical: 0,
            horizontal: rect.width.saturating_sub(27) / 2 // Adjusted for wider first column
        });

        let table = Table::new(rows, widths)
            .style(Style::new().blue())
            .header(header)
            .block(Block::new()
                .title("Soft Hands")
                .style(Style::new().bold())
                .title_alignment(Alignment::Center))
            .row_highlight_style(Style::new().reversed())
            .column_highlight_style(Style::new().red())
            .cell_highlight_style(Style::new().blue())
            .highlight_symbol(">>");

        frame.render_widget(table, inner_rect);
    }

    pub fn render_pair_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from pair hands data in the strategy
        let rows = self.strategy.tables.pair_hands.iter().map(|row| {
            // Convert pair value to string for the first column
            let pair_str = row.pair.to_string();

            // Create a vector with the pair value as the first element
            let mut row_cells = vec![pair_str];

            // Add all action codes from the row
            row_cells.extend(row.actions.iter().cloned());

            // Create a Row with all cells
            Row::new(row_cells)
        }).collect::<Vec<_>>();

        // Define column widths - one for the pair value, and one for each dealer card
        let mut widths = vec![Constraint::Length(2)]; // Width for pair value column
        widths.extend(vec![Constraint::Length(2); 10]); // Width for dealer card columns (2-A)

        // Create header cells
        let header_cells = vec![" ", "2", "3", "4", "5", "6", "7", "8", "9", "10", "A"]
            .into_iter()
            .map(|h| Cell::new(h).style(Style::new().bold()))
            .collect::<Vec<_>>();

        let header = Row::new(header_cells)
            .style(Style::new().bold())
            .bottom_margin(1)
            .top_margin(1);

        // Create inner area with margins to center the table
        let inner_rect = rect.inner(Margin {
            vertical: 0,
            horizontal: rect.width.saturating_sub(26) / 2
        });

        let table = Table::new(rows, widths)
            .style(Style::new().blue())
            .header(header)
            .block(Block::new()
                .title("Pairs")
                .style(Style::new().bold())
                .title_alignment(Alignment::Center))
            .row_highlight_style(Style::new().reversed())
            .column_highlight_style(Style::new().red())
            .cell_highlight_style(Style::new().blue())
            .highlight_symbol(">>");

        frame.render_widget(table, inner_rect);
    }
}

// ---- TRAIT IMPLEMENTATIONS ----
impl Model for StrategyCalculatorScreen {

    fn update(&mut self) -> std::io::Result<ModelResponse> {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                return Ok(ModelResponse::Refresh);
            }
            return match key.code {
                KeyCode::Char('q') => Ok(ModelResponse::Exit),
                KeyCode::Char('m') => Ok(ModelResponse::NavToMainMenu),
                // More cursor down
                KeyCode::Char('j') | KeyCode::Down => {
                    self.increment_menu_index(1);
                    return Ok(ModelResponse::Refresh);
                }
                // More cursor up
                KeyCode::Char('k') | KeyCode::Up => {
                    self.increment_menu_index(-1);
                    return Ok(ModelResponse::Refresh);
                }
                // Increment current value up
                KeyCode::Char('l') | KeyCode::Right => {
                    self.increment_current_menu_item(1);
                    self.update_strategy_based_on_settings();
                    return Ok(ModelResponse::Refresh);
                }
                // Increment current value down
                KeyCode::Char('h') | KeyCode::Left => {
                    self.increment_current_menu_item(-1);
                    self.update_strategy_based_on_settings();
                    return Ok(ModelResponse::Refresh);
                }
                _ => Ok(ModelResponse::Refresh),
            }
        }
        Ok(ModelResponse::Refresh)
    }

    fn ui(&mut self, frame: &mut Frame) {
        // First split the screen vertically to create a footer area
        let vertical = Layout::vertical([
            Constraint::Length(4),     // Buffer area,
            Constraint::Min(5),        // Main content area
            Constraint::Length(4)      // Footer area
        ]);
        let main_chunks = vertical.split(frame.area());
        let main_area = main_chunks[1];
        let footer_area = main_chunks[2];

        // Now split the main area horizontally into two sections (1/4 and 3/4)
        let horizontal_layout = Layout::horizontal([
            Constraint::Ratio(1, 4),  // Takes up 1/4 of the width (left side)
            Constraint::Ratio(3, 4),  // Takes up 3/4 of the width (right side)
        ]);
        let horiz_chunks = horizontal_layout.split(main_area);

        // Left section (1/4 of width)
        let left_section = horiz_chunks[0];
        render_border(frame, left_section);
        render_centered_text(frame, left_section, "Game Settings");

        let right_section = horiz_chunks[1];
        render_border(frame, right_section);
        render_centered_text(frame, right_section, "Strategy Chart");

        let left_vert_layout = Layout::vertical([
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Length(10),
        ]);
        let left_vert_chunks = left_vert_layout.split(left_section);

        // Split the right section (3/4 of width) into 3 vertical and 5 horizontal slices
        let right_vert_layout = Layout::vertical([
            Constraint::Length(10),
            Constraint::Min(21),
            Constraint::Length(10),
        ]);
        let right_vert_chunks = right_vert_layout.split(right_section);

        let right_layout = Layout::horizontal([
            Constraint::Length(4),         // Small buffer space
            Constraint::Ratio(1, 3),       // Equal chunk 1
            Constraint::Ratio(1, 3),       // Equal chunk 2
            Constraint::Ratio(1, 3),       // Equal chunk 3
            Constraint::Length(4),         // Small buffer space
        ]);
        let right_chunks = right_layout.split(right_vert_chunks[1]);

        self.render_menu_body(frame, left_vert_chunks[1]);
        self.render_hard_hands_table(frame, right_chunks[1]);
        self.render_soft_hands_table(frame, right_chunks[2]);
        self.render_pair_hands_table(frame, right_chunks[3]);
    }
}

impl MenuNavigation for StrategyCalculatorScreen {
    fn get_menu_length(&self) -> usize {
        ADJUSTABLE_OPTIONS.len()
    }

    fn get_menu_index(&self) -> i8 {
        self.active_menu_index
    }

    fn set_menu_index(&mut self, index: i8) {
        self.active_menu_index = index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_strategy_key() {
        // Test case 1: All options enabled
        let key1 = StrategyCalculatorScreen::create_strategy_key(
            2, true, true, true, SurrenderRule::NotAllowed
        );
        assert_eq!(key1, "decks2_s17y_dasy_peaky_surry");

        // Test case 2: All options disabled
        let key2 = StrategyCalculatorScreen::create_strategy_key(
            1, false, false, false, SurrenderRule::NotAllowed
        );
        assert_eq!(key2, "decks1_s17n_dasn_peakn_surrn");

        // Test case 3: Mixed options
        let key3 = StrategyCalculatorScreen::create_strategy_key(
            6, true, false, true, SurrenderRule::NotAllowed
        );
        assert_eq!(key3, "decks6_s17y_dasn_peaky_surry");

        // Test case 4: Different deck counts
        let key4 = StrategyCalculatorScreen::create_strategy_key(
            4, false, true, false, SurrenderRule::NotAllowed
        );
        assert_eq!(key4, "decks4_s17n_dasy_peakn_surrn");

        // Test case 5: Different surrender types
        let key5 = StrategyCalculatorScreen::create_strategy_key(
            1, true, true, true, SurrenderRule::NotAllowed
        );
        assert_eq!(key5, "decks1_s17y_dasy_peaky_surry");
    }
}