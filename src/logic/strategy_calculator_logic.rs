use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::logic::strategy_calculator_logic::SurrenderRule::{AnyUpcard, Dealer2Through10, NotAllowed};

/// Represents a complete "basic" blackjack strategy without deviations.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlackjackStrategy {
    /// Unique identifier of the strategy
    pub id: Uuid,

    /// Name of the strategy
    pub name: String,

    /// Description of strategy
    pub description: String,

    /// Rules that the strategy is designed for
    pub rules: StrategyVariables,

    /// Strategy Tables, one for Hard Hands, one for Soft Hands, and a final for Pairs
    pub tables: StrategyTables,

    /// Legend explaining the tables symbols and their corresponding actions
    pub action_legend: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyVariables {
    /// Number of decks
    pub decks: u8,

    /// Whether dealer stands on soft 17
    pub dealer_stands_on_soft_17: bool,

    /// Whether play may double after splitting
    pub double_after_split: bool,

    /// Whether dealer peaks for blackjack
    pub dealer_peak: bool,

    /// Type of surrender allowed
    pub surrender_allowed: SurrenderRule,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategyTables {
    /// Hard hand strategies (when play has no Ace in hand)
    pub hard_hands: Vec<HardHandRow>,

    /// Soft hand strategies (when player has an Ace in hand)
    pub soft_hands: Vec<SoftHandRow>,

    /// Pair splitting strategies
    pub pair_hands: Vec<PairRow>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardHandRow {
    /// Total value of player's hand (5-21)
    pub total: u8,
    /// Actions to take based on dealer's upcard (2,A)
    /// Index 0 = dealer's 2, index 9 = dealer's A
    pub actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoftHandRow {
    /// Total value of player's hand (13-21)
    pub total: u8,
    /// Actions to take based on dealer's upcard (2,A)
    /// Index 0 = dealer's 2, index 9 = dealer's A
    pub actions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairRow {
    /// Total value of player's hand (2-11)
    pub pair: u8,
    /// Actions to take based on dealer's upcard (2,A)
    /// Index 0 = dealer's 2, index 9 = dealer's A
    pub actions: Vec<String>,
}

#[derive(Serialize, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurrenderRule {
    NotAllowed,
    AnyUpcard,
    Dealer2Through10,
}

// Add this implementation to your SurrenderRule enum
impl<'de> Deserialize<'de> for SurrenderRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SurrenderRule::from_string(&s).map_err(serde::de::Error::custom)
    }
}

impl SurrenderRule {
    pub fn from_string(str_value: &str) -> Result<Self, String> {
        match str_value {
            "Not Allowed" => Ok(NotAllowed),
            "Any Dealer Upcard" => Ok(AnyUpcard),
            "Dealer 2 through 10" => Ok(Dealer2Through10),
            _ => Err(format!("Unknown Surrender Rule: {}", str_value))
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            NotAllowed => "Not Allowed",
            AnyUpcard => "Any Dealer Upcard",
            Dealer2Through10 => "Dealer 2 through 10",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    Hit,
    Stand,
    DoubleOrHit,
    DoubleOrStand,
    Split,
    SurrenderOrHit,
    SurrenderOrStand,
    SplitOrHit,
}

/// Helper methods to convert between action codes (as Strings) and PlayerActions
impl PlayerAction {
    /// Convert a string action code to a PlayerAction
    pub fn from_code_string(code: &str) -> Result<Self, String> {
        match code {
            "H" => Ok(PlayerAction::Hit),
            "S" => Ok(PlayerAction::Stand),
            "D" => Ok(PlayerAction::DoubleOrHit),
            "Ds" => Ok(PlayerAction::DoubleOrStand),
            "P" => Ok(PlayerAction::Split),
            "Su" => Ok(PlayerAction::SurrenderOrHit),
            "Rs" => Ok(PlayerAction::SurrenderOrStand),
            "Ph" => Ok(PlayerAction::SplitOrHit),
            _ => Err(format!("Unknown action code: {}", code)),
        }
    }

    /// Convert PlayerAction to a string action code
    pub fn to_code_string(&self) -> &'static str {
        match self {
            PlayerAction::Hit => "H",
            PlayerAction::Stand => "S",
            PlayerAction::DoubleOrHit => "D",
            PlayerAction::DoubleOrStand => "Ds",
            PlayerAction::Split => "P",
            PlayerAction::SurrenderOrHit => "Su",
            PlayerAction::SurrenderOrStand => "Rs",
            PlayerAction::SplitOrHit => "Ph",
        }
    }
}

impl BlackjackStrategy {
    /// Create a new BlackjackStrategy with default values
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Default Strategy".to_string(),
            description: "Default Basic Strategy".to_string(),
            rules: StrategyVariables {
                decks: 1,
                dealer_stands_on_soft_17: true,
                double_after_split: true,
                dealer_peak: true,
                surrender_allowed: NotAllowed
            },
            tables: StrategyTables {
                hard_hands: Vec::new(),
                soft_hands: Vec::new(),
                pair_hands: Vec::new(),
            },
            action_legend: HashMap::new(),
        }
    }

    /// Parse a BlackjackStrategy from a JSON string
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let mut strategy: Self = serde_json::from_str(json_str)?;
        if strategy.id == Uuid::nil() {
            strategy.id = Uuid::new_v4();
        }
        Ok(strategy)
    }

    /// Load a BlackjackStrategy from a file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(path)?;
        let strategy = Self::from_json(&file_content)?;
        Ok(strategy)
    }

    pub fn get_hard_hand_action(&self, total: u8, dealer_upcard: u8) -> Result<PlayerAction, String> {
        if !(5..=21).contains(&total) {
            return Err(format!("Invalid hard hand total: {}", total));
        }

        if !(2..=11).contains(&dealer_upcard) {
            return Err(format!("Invalid dealer upcard: {}", dealer_upcard));
        }

        // Find the matching row
        let row = self.tables.hard_hands.iter()
            .find(|row| row.total == total)
            .ok_or_else(|| format!("No strategy for hard hand with total {}", total))?;

        // Get the correct action based on dealer's upcard.
        // Dealer's 2 maps to index 0. Dealers A maps to index 9.
        let index = (dealer_upcard - 2) as usize;

        let action_code = row.actions.get(index)
            .ok_or_else(|| format!("Missing action for dealer upcard {} in hand total {}", dealer_upcard, total))?;

        PlayerAction::from_code_string(action_code)
    }

    pub fn get_soft_hand_action(&self, total: u8, dealer_upcard: u8) -> Result<PlayerAction, String> {
        Err("Not implemented".to_string())
    }

    pub fn get_pair_action(&self, single_card_value: u8, dealer_upcard: u8) -> Result<PlayerAction, String> {
        Err("Not implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_minimal() {
        let json_str = r#"{
            "id": "827396a6-eaa0-4dc3-986b-926fa0fcf798",
            "name": "Test Strategy",
            "description": "For Testing",
            "rules": {
                "decks": 1,
                "dealer_stands_on_soft_17": true,
                "double_after_split": true,
                "dealer_peak": true,
                "surrender_allowed": "Not Allowed"
            },
            "tables": {
                "hard_hands": [],
                "soft_hands": [],
                "pair_hands": []
            },
            "action_legend": {}
        }"#;

        let result = BlackjackStrategy::from_json(json_str);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.description, "For Testing");
        assert_eq!(strategy.rules.decks, 1);
        assert_eq!(strategy.rules.dealer_stands_on_soft_17, true);
        assert_eq!(strategy.tables.hard_hands.len(), 0);
        assert!(!strategy.id.is_nil()); // Should have generated a UUID
        println!("{:#?}", strategy)
    }

    #[test]
    fn test_from_json_with_tables() {
        let json_str = r#"{
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "Test Strategy",
            "description": "For Testing",
            "rules": {
                "decks": 2,
                "dealer_stands_on_soft_17": false,
                "double_after_split": true,
                "dealer_peak": true,
                "surrender_allowed": "Not Allowed"
            },
            "tables": {
                "hard_hands": [
                    { "total": 10, "actions": ["H", "D", "D", "D", "D", "D", "D", "D", "H", "H"] }
                ],
                "soft_hands": [
                    { "total": 18, "actions": ["S", "D", "D", "D", "D", "S", "S", "H", "H", "S"] }
                ],
                "pair_hands": [
                    { "pair": 7, "actions": ["P", "P", "P", "P", "P", "P", "P", "H", "S", "H"] }
                ]
            },
            "action_legend": {
                "H": "Hit",
                "S": "Stand",
                "D": "Double",
                "P": "Split"
            }
        }"#;

        let result = BlackjackStrategy::from_json(json_str);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.rules.decks, 2);
        assert_eq!(strategy.rules.dealer_stands_on_soft_17, false);

        // Verify tables were parsed correctly
        assert_eq!(strategy.tables.hard_hands.len(), 1);
        assert_eq!(strategy.tables.hard_hands[0].total, 10);
        assert_eq!(strategy.tables.hard_hands[0].actions.len(), 10);
        assert_eq!(strategy.tables.hard_hands[0].actions[0], "H");

        assert_eq!(strategy.tables.soft_hands.len(), 1);
        assert_eq!(strategy.tables.soft_hands[0].total, 18);

        assert_eq!(strategy.tables.pair_hands.len(), 1);
        assert_eq!(strategy.tables.pair_hands[0].pair, 7);

        // Verify action legend was parsed
        assert_eq!(strategy.action_legend.len(), 4);
        assert_eq!(strategy.action_legend.get("H").unwrap(), "Hit");
    }

    #[test]
    fn test_get_hard_hand_action() {
        let json_str = r#"{
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "Test Strategy",
            "description": "Test Strategy",
            "rules": {
                "decks": 1,
                "dealer_stands_on_soft_17": true,
                "double_after_split": true,
                "dealer_peak": true,
                "surrender_allowed": "Not Allowed"
            },
            "tables": {
                "hard_hands": [
                    { "total": 10, "actions": ["D", "D", "D", "D", "D", "D", "D", "D", "H", "H"] },
                    { "total": 16, "actions": ["S", "S", "S", "S", "S", "H", "H", "H", "H", "H"] }
                ],
                "soft_hands": [],
                "pair_hands": []
            },
            "action_legend": {
                "H": "Hit",
                "S": "Stand",
                "D": "Double if allowed, otherwise Hit"
            }
        }"#;

        let strategy = BlackjackStrategy::from_json(json_str).unwrap();

        // Test valid inputs
        let action10vs2 = strategy.get_hard_hand_action(10, 2).unwrap();
        assert_eq!(action10vs2, PlayerAction::DoubleOrHit);

        let action16vs6 = strategy.get_hard_hand_action(16, 6).unwrap();
        assert_eq!(action16vs6, PlayerAction::Stand);

        let action16vs5 = strategy.get_hard_hand_action(16, 7).unwrap();
        assert_eq!(action16vs5, PlayerAction::Hit);

        // Test invalid hand total
        let result = strategy.get_hard_hand_action(4, 2);
        assert!(result.is_err());

        // Test invalid dealer upcard
        let result = strategy.get_hard_hand_action(10, 1);
        assert!(result.is_err());

        // Test dealer Ace (value 11)
        let action10vsA = strategy.get_hard_hand_action(10, 11).unwrap();
        assert_eq!(action10vsA, PlayerAction::Hit);
    }

    #[test]
    fn test_load_default_strategy() {
        // Assuming the default-strategy.json is in the root of your project
        // You may need to adjust the path based on your project structure
        let result = BlackjackStrategy::from_file("/Users/atticus/WizardsWorkshop/FreesideProjects/jacks-blackjack/resources/strategies/default-strategy.json");
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.name, "Basic Strategy");
        assert_eq!(strategy.description, "Default Basic Strategy");
        assert_eq!(strategy.rules.decks, 2);
        assert_eq!(strategy.rules.dealer_stands_on_soft_17, false);

        // Check that tables were loaded properly
        assert_eq!(strategy.tables.hard_hands.len(), 17);
        assert_eq!(strategy.tables.soft_hands.len(), 9);
        assert_eq!(strategy.tables.pair_hands.len(), 10);

        // Check a specific strategy recommendation
        // Hard 16 vs Dealer 10 should be Hit (H)
        let hard16_vs10 = strategy.get_hard_hand_action(16, 10).unwrap();
        assert_eq!(hard16_vs10, PlayerAction::Hit);

        // Hard 11 vs any card should be Double (D)
        let hard11_vs7 = strategy.get_hard_hand_action(11, 7).unwrap();
        assert_eq!(hard11_vs7, PlayerAction::DoubleOrHit);

        // Check that we can also retrieve the action legend
        assert_eq!(strategy.action_legend.len(), 8);
        assert_eq!(strategy.action_legend.get("H").unwrap(), "Hit");
        assert_eq!(strategy.action_legend.get("S").unwrap(), "Stand");
        assert_eq!(strategy.action_legend.get("D").unwrap(), "Double if allowed, otherwise Hit");
    }
}