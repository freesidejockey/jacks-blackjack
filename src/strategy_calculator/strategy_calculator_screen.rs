use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
use crate::model::{Model, ModelResponse};
use crate::ui::{create_common_layout, create_header_main_footer_layout, render_border, render_centered_text, render_footer_spans, split_content_horizontally, MenuNavigation};
use color_eyre::owo_colors::OwoColorize;
use fakeit::name::first;
use itertools::Itertools;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::style::{Color, Style, Styled};
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
        let default_decks = 3;
        let default_dealer_stands_on_soft_17 = true;
        let default_double_after_split = true;
        let default_surrender = SurrenderRule::AnyUpcard;
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
                                panic!()
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
            number_of_decks: default_decks as i8,
            dealer_stands_on_soft_17: default_dealer_stands_on_soft_17,
            allow_double_after_split: default_double_after_split,
            surrender_rule: default_surrender,
            dealer_peak: default_dealer_peak,
            strategy: default_strategy,
            strategy_cache,
            active_strategy_name,
        }
    }

    fn get_action_color(&self, action: &str) -> Color {
        match action.trim() {
            "H" => Color::Red,
            "D" | "Dh" => Color::Blue,
            "Ds" => Color::LightBlue,
            "S" => Color::Yellow,
            "P" => Color::LightCyan,
            "Su" | "Rs" | "Rp" => Color::LightMagenta,
            "Rh" => Color::Magenta,
             _ => Color::Red
        }
    }

    fn create_colored_row<'a>(&self, row_data: Vec<String>) -> Row<'a> {
        let first_cell = Cell::new(row_data[0].clone());

        let mut cells = vec![first_cell];
        for action in row_data.iter().skip(1) {
            let color = self.get_action_color(action);
            cells.push(Cell::new(action.clone()).style(Style::new().fg(color)));
        }

        Row::new(cells)
    }

    // Static method that doesn't require &self
    pub fn find_matching_strategy(
        strategy_cache: &HashMap<String, BlackjackStrategy>,
        decks: u8,
        dealer_stands_on_soft_17: bool,
        double_after_split: bool,
        dealer_peak: bool,
        surrender_rule: SurrenderRule
    ) -> Option<(String, &BlackjackStrategy)> {
        // Iterate through all cached strategies
        for (name, strategy) in strategy_cache {
            // Check for exact match on all variables
            if strategy.rules.decks == decks &&
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
        if let Some((name, matching_strategy)) = Self::find_matching_strategy(
            &self.strategy_cache,
            decks,
            dealer_stands_on_soft_17,
            double_after_split,
            dealer_peak,
            surrender_allowed
        ) {
            // Update the active strategy if we found a match
            self.switch_strategy(&*name);
        } else {
            if self.strategy_cache.contains_key("default-strategy.json") {
                self.switch_strategy("default-strategy");
            }

            // You might want to log this missing combination for future strategy creation:
            let key = Self::create_strategy_key(
                decks,
                dealer_stands_on_soft_17,
                double_after_split,
                dealer_peak,
                surrender_allowed
            );
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
                0 => text.push_str(&format!(": < {} >", match self.number_of_decks {
                    1 => "1",
                    2 => "2",
                    3 => "4+",
                    _ => "Unknown"
                })),
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


    fn render_action_legend(&self, frame: &mut Frame, rect: Rect) {
        let mut strat_key_lines: Vec<Line<'_>> = vec![];

        let main = rect.inner(Margin {
            vertical: 0,
            horizontal: (rect.width.saturating_sub(40) / 2)
        });

        let vert_split = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(10)
        ]).split(main);

        // Render key header
        let header_sect = vert_split[0];
        let header = Paragraph::new("Action Legend")
            .bold()
            .alignment(Alignment::Center)
            .block(Block::default());

        frame.render_widget(header, header_sect);

        // Render key body
        let body_sect = vert_split[1];

        // Create a sorted collection of the action legend items
        // First, collect into a Vec to allow sorting
        let mut sorted_legend: Vec<(&String, &String)> = self.strategy.action_legend.iter().collect();

        // Sort by the action code alphabetically
        sorted_legend.sort_by(|a, b| a.0.cmp(b.0));

        // Now create the styled lines in alphabetical order
        for (code, description) in sorted_legend {
            strat_key_lines.push(
                Line::from(
                    format!("{}: {}", code, description)).fg(self.get_action_color(code)));
        }

        // Render
        let key = Paragraph::new(strat_key_lines)
            .bold()
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(key, body_sect);
    }

    fn increment_current_menu_item(&mut self, increment: i8) {
        let menu_item = ADJUSTABLE_OPTIONS.get(self.active_menu_index as usize).unwrap();
        match menu_item {
            AdjustableOption::NumberOfDecks => {
                // Bound the deck count between 1 and 6
                self.number_of_decks = (self.number_of_decks + increment).clamp(1, 3);
            }
            AdjustableOption::Soft17DealerAction => {
                // Simple boolean toggle
                self.dealer_stands_on_soft_17 = !self.dealer_stands_on_soft_17;
            }
            AdjustableOption::AllowDoubleAfterSplit => {
                // Simple boolean toggle
                self.allow_double_after_split = !self.allow_double_after_split;
            }
            AdjustableOption::AllowSurrender => {
                // Cycle through surrender rules and increment combo
                self.surrender_rule = match (self.surrender_rule, increment > 0) {
                    (SurrenderRule::NotAllowed, true) => SurrenderRule::AnyUpcard,
                    (SurrenderRule::AnyUpcard, true) => SurrenderRule::Dealer2Through10,
                    (SurrenderRule::Dealer2Through10, true) => SurrenderRule::NotAllowed,
                    (SurrenderRule::Dealer2Through10, false) => SurrenderRule::AnyUpcard,
                    (SurrenderRule::AnyUpcard, false) => SurrenderRule::NotAllowed,
                    (SurrenderRule::NotAllowed, false) => SurrenderRule::Dealer2Through10,
                }
            }
            AdjustableOption::DealerPeak => {
                self.dealer_peak = !self.dealer_peak;
            }
        }
    }

    // Modified table rendering methods
    pub fn render_hard_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from hard hands data with conditional coloring
        let rows = self.strategy.tables.hard_hands.iter().map(|row| {
            let mut row_cells = vec![row.total.to_string()];
            row_cells.extend(row.actions.iter().cloned());
            self.create_colored_row(row_cells)
        }).collect::<Vec<_>>();

        // Create a table with consistent styling
        let widths = self.create_table_column_constraints(3);
        let table = self.create_strategy_table(rows, widths, "Hard Hands");

        // Render in a centered area
        let inner_rect = self.create_centered_table_area(rect, 26);
        frame.render_widget(table, inner_rect);
    }

    pub fn render_soft_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from soft hands data
        let rows = self.strategy.tables.soft_hands.iter().map(|row| {
            let second_card = row.total - 11;
            let hand_display = format!("A{}", second_card);

            let mut row_cells = vec![hand_display];
            row_cells.extend(row.actions.iter().cloned());
            self.create_colored_row(row_cells)
        }).collect::<Vec<_>>();

        // Create a table with consistent styling
        let widths = self.create_table_column_constraints(3); // Wider first column for A+X format
        let table = self.create_strategy_table(rows, widths, "Soft Hands");

        // Render in a centered area
        let inner_rect = self.create_centered_table_area(rect, 27); // 27 for wider first column
        frame.render_widget(table, inner_rect);
    }

    pub fn render_pair_hands_table(&mut self, frame: &mut Frame, rect: Rect) {
        // Create rows from pair hands data
        let rows = self.strategy.tables.pair_hands.iter().map(|row| {
            let mut row_cells = vec![row.pair.to_string()];
            row_cells.extend(row.actions.iter().cloned());
            self.create_colored_row(row_cells)
        }).collect::<Vec<_>>();

        // Create a table with consistent styling
        let widths = self.create_table_column_constraints(3);
        let table = self.create_strategy_table(rows, widths, "Pairs");

        // Render in a centered area
        let inner_rect = self.create_centered_table_area(rect, 26);
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
        // Create main vertical layout
        let main_chunks = create_common_layout(frame.area());
        let main_area = main_chunks[1];

        // Now split the main area horizontally into two sections (1/4 and 3/4)
        let horizontal_chunks = split_content_horizontally(main_area);

        // Render the settings Section
        let left_section = horizontal_chunks[0];
        render_border(frame, left_section);
        render_centered_text(frame, left_section, " Game Settings ");


        let left_section_chunks = Self::create_header_main_main_footer_layout(left_section, 10, 20, 10);
        let menu_rect = left_section_chunks[1];
        let strategy_key_rect = left_section_chunks[3];


        self.render_menu_body(frame, menu_rect);
        self.render_action_legend(frame, strategy_key_rect);

        // Render the Strategy Tables
        let right_section = horizontal_chunks[1];
        render_border(frame, right_section);
        render_centered_text(frame, right_section, " Strategy Chart ");

        let tables_rect =
            create_header_main_footer_layout(right_section, 10, 21, 10)[1];

        let right_layout = Layout::horizontal([
            Constraint::Length(4),         // Small buffer space
            Constraint::Ratio(1, 3),       // Equal chunk 1
            Constraint::Ratio(1, 3),       // Equal chunk 2
            Constraint::Ratio(1, 3),       // Equal chunk 3
            Constraint::Length(4),         // Small buffer space
        ]);
        let right_chunks = right_layout.split(tables_rect);

        self.render_hard_hands_table(frame, right_chunks[1]);
        self.render_soft_hands_table(frame, right_chunks[2]);
        self.render_pair_hands_table(frame, right_chunks[3]);

        // Render Footer
        let footer_area = main_chunks[2];

        let footer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(20),
            ])
            .split(footer_area);
        render_footer_spans(frame, vec![], footer_layout[1]);
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

impl StrategyCalculatorScreen {
    // New helper methods for table styling

    /// Creates a consistently styled header row for strategy tables
    fn create_table_header(&self) -> Row<'static> {
        let header_cells = vec![" ", "2", "3", "4", "5", "6", "7", "8", "9", "10", "A"]
            .into_iter()
            .map(|h| Cell::new(h).style(Style::new().bold()))
            .collect::<Vec<_>>();

        Row::new(header_cells)
            .style(Style::new().bold())
            .bottom_margin(1)
            .top_margin(1)
    }

    /// Creates column constraints for strategy tables
    /// first_col_width: width for the first column (hand description)
    fn create_table_column_constraints(&self, first_col_width: u16) -> Vec<Constraint> {
        let mut constraints = vec![Constraint::Length(first_col_width)]; // First column (hand type)
        constraints.extend(vec![Constraint::Length(2); 10]);
        constraints
    }

    /// Creates a styled table with the standard layout and styling
    fn create_strategy_table<'a>(
        &self,
        rows: Vec<Row<'a>>,
        widths: Vec<Constraint>,
        title: &'a str
    ) -> Table<'a> {
        Table::new(rows, widths)
            .style(Style::new().blue())
            .header(self.create_table_header())
            .block(Block::new()
                .title(title)
                .style(Style::new().bold())
                .title_alignment(Alignment::Center))
            .row_highlight_style(Style::new().reversed())
            .column_spacing(1)
            .column_highlight_style(Style::new().red())
            .cell_highlight_style(Style::new().blue())
            .highlight_symbol(">>")
    }

    /// Creates a centered inner area for a table with appropriate margins
    fn create_centered_table_area(&self, rect: Rect, table_width: u16) -> Rect {
        rect.inner(Margin {
            vertical: 0,
            horizontal: (rect.width.saturating_sub(table_width) / 2).saturating_sub(5)
        })
    }

    pub fn create_header_main_main_footer_layout(area: Rect,
                                            header_height: u16,
                                            main_height: u16,
                                            footer_height: u16) -> Rc<[Rect]> {
        Layout::vertical([
            Constraint::Length(header_height),
            Constraint::Min(main_height/2),
            Constraint::Length(2),
            Constraint::Min(main_height/2),
            Constraint::Length(footer_height),
        ]).split(area)
    }
}