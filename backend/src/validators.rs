use regex::Regex;
use crate::errors::AppError;

pub fn validate_username(username: &String) -> Result<(), AppError> {
    if username.is_empty() {
        return Err(AppError::BadRequest("Username is required".to_string()));
    }

    if username.len() < 3 {
        return Err(AppError::BadRequest(
            "Username cannot be smaller than 3 characters".to_string(),
        ));
    }

    if username.len() > 25 {
        return Err(AppError::BadRequest(
            "Username cannot be greater than 25 characters".to_string(),
        ));
    }


    /*
     * At least 1 letter.
     * Starts with a letter or a digit.
     * 3-25 characters.
     * Only letters, digits, hyphen and underscores are allowed.
     * Hyphen and underscores have to be followed by a letter or a digit.
     */
    let chars: Vec<char> = username.chars().collect();

    // Must start with letter or digit
    if !chars[0].is_ascii_alphanumeric() {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // At least 1 letter
    if !chars.iter().any(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Allowed characters only
    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
    {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    // Hyphen/underscore must be followed by a letter/digit
    for w in chars.windows(2) {
        if (w[0] == '-' || w[0] == '_') && !w[1].is_ascii_alphanumeric() {
            return Err(AppError::BadRequest(
                "Must be a valid username".to_string(),
            ));
        }
    }

    if chars.last() == Some(&'-') || chars.last() == Some(&'_') {
        return Err(AppError::BadRequest(
            "Must be a valid username".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_email(email: &String) -> Result<(), AppError> {
    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    if email.len() > 255 {
        return Err(AppError::BadRequest(
            "Email cannot be greater than 255 characters".to_string(),
        ));
    }

    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !re.is_match(&email) {
        return Err(AppError::BadRequest("Must be a valid email".to_string()));
    }
    Ok(())
}

pub fn validate_password(password: &String) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::BadRequest("Password is required".to_string()));
    }

    if password.len() < 12 {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(AppError::BadRequest(
            "Password cannot be longer than 128 characters".to_string(),
        ));
    }

    let has_lower   = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper   = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit   = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| r"!@#$%^&*()_-+=[]{};:,.<>?/~\|".contains(c));

    let categories = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();

    if categories < 3 {
        return Err(AppError::BadRequest(
                "Password must contain at least 3 of these 4 : lowercase, uppercase, digit, special character".to_string()));
    }

    if password.chars()
        .collect::<Vec<_>>()
        .windows(4)
        .any(|w| w.iter().all(|&c| c == w[0]))
    {
        return Err(AppError::BadRequest(
                "Password cannot contain 4 identical characters in a row".to_string()));
    }

    if password.trim() != password {
        return Err(AppError::BadRequest(
                "Password cannot start or end with whitespace".to_string()));
    }

    // TODO: blacklist common passwords
    Ok(())
}

pub fn validate_strategy(strat: &StrategyContent) -> Result<(), AppError> {
    let validator = StrategyValidator::new(
        vec![
            "sma_10".to_string(),
            "sma_50".to_string(),
            "rsi".to_string(),
            "macd".to_string(),
            "volume".to_string(),
        ],
        vec!["BUY".to_string(), "SELL".to_string(), "HOLD".to_string()],
    );

    let _analysis = validator.validate_and_analyze(&strat).unwrap();
    Ok(())
}



use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// PHILOSOPHY: Enable Creativity, Control Cost
// ============================================================================
// Instead of limiting WHAT users can do, we:
// 1. Let them build ANY strategy structure
// 2. Validate it's SAFE (no injection, valid syntax)
// 3. Calculate its COST (computational complexity)
// 4. Enforce BUDGET (credits/quotas based on cost)
// 5. Execute SAFELY (isolated, timeout, resource limits)
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndicatorField {
    pub name: String,
    pub category: IndicatorCategory,
    pub description: String,
    pub data_type: DataType,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum IndicatorCategory {
    Price,        // open, close, high, low
    Volume,       // volume, dollar_volume
    Volatility,   // atr, bollinger_bands
    Trend,        // sma, ema, macd, adx
    Momentum,     // rsi, stochastic, cci
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum DataType {
    Float,
    Integer,
    Boolean,
}

// ============================================================================
// FLEXIBLE STRATEGY DEFINITION
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Value {
    Field(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "op")]
pub enum Condition {
    // Comparison operators
    #[serde(rename = "gt")]
    GreaterThan { left: Box<Value>, right: Box<Value> },
    
    #[serde(rename = "lt")]
    LessThan { left: Box<Value>, right: Box<Value> },
    
    #[serde(rename = "gte")]
    GreaterThanOrEqual { left: Box<Value>, right: Box<Value> },
    
    #[serde(rename = "lte")]
    LessThanOrEqual { left: Box<Value>, right: Box<Value> },
    
    #[serde(rename = "eq")]
    Equal { left: Box<Value>, right: Box<Value> },
    
    #[serde(rename = "neq")]
    NotEqual { left: Box<Value>, right: Box<Value> },
    
    // Logical operators
    #[serde(rename = "and")]
    And { conditions: Vec<Condition> },
    
    #[serde(rename = "or")]
    Or { conditions: Vec<Condition> },
    
    #[serde(rename = "not")]
    Not { condition: Box<Condition> },
    
    // Advanced operators
    #[serde(rename = "bet")]
    Between { value: Box<Value>, min: f64, max: f64 },
    
    #[serde(rename = "c_ab")]
    CrossesAbove { series1: String, series2: String },
    
    #[serde(rename = "c_be")]
    CrossesBelow { series1: String, series2: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Action {
    Simple(String),        // "BUY", "SELL", "HOLD"
    Weighted {             // Position sizing
        action: String,
        weight: f64,       // 0.0 to 1.0
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Branch {
    Action(Action),
    Nested(Box<StrategyContent>),
}

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct StrategyContent {
    pub condition: Condition,
    pub then: Branch,
    #[serde(rename = "else")]
    pub else_branch: Branch,
}

// ============================================================================
// COST-BASED VALIDATION
// ============================================================================

#[derive(Debug, Clone)]
pub struct CostAnalysis {
    pub total_cost: u64,
    pub node_count: usize,
    pub max_depth: usize,
    pub indicator_count: usize,
    pub breakdown: CostBreakdown,
}

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub comparisons: u64,
    pub logical_ops: u64,
    pub advanced_ops: u64,
    pub branches: u64,
}

pub struct StrategyValidator {
    available_indicators: HashSet<String>,
    allowed_actions: HashSet<String>,
    cost_config: CostConfig,
}

#[derive(Debug, Clone)]
pub struct CostConfig {
    pub cost_per_comparison: u64,
    pub cost_per_logical_op: u64,
    pub cost_per_advanced_op: u64,  // crosses, etc.
    pub cost_per_branch: u64,
    pub cost_per_indicator: u64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            cost_per_comparison: 1,
            cost_per_logical_op: 2,
            cost_per_advanced_op: 5,
            cost_per_branch: 3,
            cost_per_indicator: 2,
        }
    }
}

impl StrategyValidator {
    pub fn new(available_indicators: Vec<String>, allowed_actions: Vec<String>) -> Self {
        Self {
            available_indicators: available_indicators.into_iter().collect(),
            allowed_actions: allowed_actions.into_iter().collect(),
            cost_config: CostConfig::default(),
        }
    }

    /// Main validation entry point
    pub fn validate_and_analyze(&self, strategy: &StrategyContent) -> Result<CostAnalysis, String> {
        // Step 1: Security validation (no injection, valid fields)
        self.validate_security(strategy)?;
        
        // Step 2: Analyze computational cost
        let cost_analysis = self.analyze_cost(strategy);
        
        Ok(cost_analysis)
    }

    /// Security validation - ensures no malicious content
    fn validate_security(&self, strategy: &StrategyContent) -> Result<(), String> {
        self.validate_condition_security(&strategy.condition)?;
        self.validate_branch_security(&strategy.then)?;
        self.validate_branch_security(&strategy.else_branch)?;
        Ok(())
    }

    fn validate_condition_security(&self, condition: &Condition) -> Result<(), String> {
        match condition {
            Condition::GreaterThan { left, right }
            | Condition::LessThan { left, right }
            | Condition::GreaterThanOrEqual { left, right }
            | Condition::LessThanOrEqual { left, right }
            | Condition::Equal { left, right }
            | Condition::NotEqual { left, right } => {
                self.validate_value_security(left)?;
                self.validate_value_security(right)?;
            }
            Condition::And { conditions } | Condition::Or { conditions } => {
                if conditions.is_empty() {
                    return Err("Empty logical operator".to_string());
                }
                for cond in conditions {
                    self.validate_condition_security(cond)?;
                }
            }
            Condition::Not { condition } => {
                self.validate_condition_security(condition)?;
            }
            Condition::Between { value, min, max } => {
                self.validate_value_security(value)?;
                if !min.is_finite() || !max.is_finite() {
                    return Err("Invalid min/max values".to_string());
                }
                if min >= max {
                    return Err("min must be less than max".to_string());
                }
            }
            Condition::CrossesAbove { series1, series2 }
            | Condition::CrossesBelow { series1, series2 } => {
                self.validate_field_name(series1)?;
                self.validate_field_name(series2)?;
            }
        }
        Ok(())
    }

    fn validate_value_security(&self, value: &Value) -> Result<(), String> {
        match value {
            Value::Field(field) => self.validate_field_name(field),
            Value::Number(n) => {
                if !n.is_finite() {
                    return Err("Invalid number (NaN or Infinity)".to_string());
                }
                Ok(())
            }
            Value::Boolean(_) => Ok(()),
        }
    }

    fn validate_field_name(&self, field: &str) -> Result<(), String> {
        // Security: alphanumeric + underscore only
        if !field.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Invalid field name: {}", field));
        }

        // Check if field exists in available indicators
        if !self.available_indicators.contains(field) {
            return Err(format!("Unknown indicator: {}", field));
        }

        Ok(())
    }

    fn validate_branch_security(&self, branch: &Branch) -> Result<(), String> {
        match branch {
            Branch::Action(action) => {
                let action_name = match action {
                    Action::Simple(name) => name,
                    Action::Weighted { action, weight } => {
                        if *weight < 0.0 || *weight > 1.0 {
                            return Err("Weight must be between 0.0 and 1.0".to_string());
                        }
                        action
                    }
                };
                
                if !self.allowed_actions.contains(action_name) {
                    return Err(format!("Invalid action: {}", action_name));
                }
            }
            Branch::Nested(strategy) => {
                self.validate_security(strategy)?;
            }
        }
        Ok(())
    }

    /// Cost analysis - calculates computational complexity
    fn analyze_cost(&self, strategy: &StrategyContent) -> CostAnalysis {
        let mut breakdown = CostBreakdown {
            comparisons: 0,
            logical_ops: 0,
            advanced_ops: 0,
            branches: 0,
        };

        let mut indicators_used = HashSet::new();
        let max_depth = self.calculate_depth(strategy, 0);

        self.count_costs(&strategy.condition, &mut breakdown, &mut indicators_used);
        self.count_branch_costs(&strategy.then, &mut breakdown, &mut indicators_used);
        self.count_branch_costs(&strategy.else_branch, &mut breakdown, &mut indicators_used);

        let total_cost = breakdown.comparisons * self.cost_config.cost_per_comparison
            + breakdown.logical_ops * self.cost_config.cost_per_logical_op
            + breakdown.advanced_ops * self.cost_config.cost_per_advanced_op
            + breakdown.branches * self.cost_config.cost_per_branch
            + (indicators_used.len() as u64) * self.cost_config.cost_per_indicator;

        let node_count = (breakdown.comparisons + breakdown.logical_ops 
                         + breakdown.advanced_ops + breakdown.branches) as usize;

        CostAnalysis {
            total_cost,
            node_count,
            max_depth,
            indicator_count: indicators_used.len(),
            breakdown,
        }
    }

    fn count_costs(
        &self,
        condition: &Condition,
        breakdown: &mut CostBreakdown,
        indicators: &mut HashSet<String>,
    ) {
        match condition {
            Condition::GreaterThan { left, right }
            | Condition::LessThan { left, right }
            | Condition::GreaterThanOrEqual { left, right }
            | Condition::LessThanOrEqual { left, right }
            | Condition::Equal { left, right }
            | Condition::NotEqual { left, right } => {
                breakdown.comparisons += 1;
                self.collect_indicators(left, indicators);
                self.collect_indicators(right, indicators);
            }
            Condition::And { conditions } | Condition::Or { conditions } => {
                breakdown.logical_ops += 1;
                for cond in conditions {
                    self.count_costs(cond, breakdown, indicators);
                }
            }
            Condition::Not { condition } => {
                breakdown.logical_ops += 1;
                self.count_costs(condition, breakdown, indicators);
            }
            Condition::Between { value, .. } => {
                breakdown.comparisons += 2; // min <= value <= max
                self.collect_indicators(value, indicators);
            }
            Condition::CrossesAbove { series1, series2 }
            | Condition::CrossesBelow { series1, series2 } => {
                breakdown.advanced_ops += 1;
                indicators.insert(series1.clone());
                indicators.insert(series2.clone());
            }
        }
    }

    fn count_branch_costs(
        &self,
        branch: &Branch,
        breakdown: &mut CostBreakdown,
        indicators: &mut HashSet<String>,
    ) {
        breakdown.branches += 1;
        if let Branch::Nested(strategy) = branch {
            self.count_costs(&strategy.condition, breakdown, indicators);
            self.count_branch_costs(&strategy.then, breakdown, indicators);
            self.count_branch_costs(&strategy.else_branch, breakdown, indicators);
        }
    }

    fn collect_indicators(&self, value: &Value, indicators: &mut HashSet<String>) {
        if let Value::Field(field) = value {
            indicators.insert(field.clone());
        }
    }

    fn calculate_depth(&self, strategy: &StrategyContent, current_depth: usize) -> usize {
        let then_depth = match &strategy.then {
            Branch::Action(_) => current_depth + 1,
            Branch::Nested(s) => self.calculate_depth(s, current_depth + 1),
        };

        let else_depth = match &strategy.else_branch {
            Branch::Action(_) => current_depth + 1,
            Branch::Nested(s) => self.calculate_depth(s, current_depth + 1),
        };

        then_depth.max(else_depth)
    }
}

// ============================================================================
// BUDGET SYSTEM
// ============================================================================

#[derive(Debug, Clone)]
pub struct UserBudget {
    pub max_cost_per_strategy: u64,
    pub max_depth: usize,
    pub max_indicators: usize,
}

impl UserBudget {
    pub fn free_tier() -> Self {
        Self {
            max_cost_per_strategy: 30,
            max_depth: 3,
            max_indicators: 10,
        }
    }

    pub fn premium_tier() -> Self {
        Self {
            max_cost_per_strategy: 500,
            max_depth: 10,
            max_indicators: 20,
        }
    }

    pub fn enterprise_tier() -> Self {
        Self {
            max_cost_per_strategy: 2000,
            max_depth: 20,
            max_indicators: 50,
        }
    }

    pub fn check_budget(&self, analysis: &CostAnalysis) -> Result<(), String> {
        if analysis.total_cost > self.max_cost_per_strategy {
            return Err(format!(
                "Strategy cost {} exceeds budget of {}. Simplify your strategy or upgrade your tier.",
                analysis.total_cost, self.max_cost_per_strategy
            ));
        }

        if analysis.max_depth > self.max_depth {
            return Err(format!(
                "Strategy depth {} exceeds maximum of {}",
                analysis.max_depth, self.max_depth
            ));
        }

        if analysis.indicator_count > self.max_indicators {
            return Err(format!(
                "Strategy uses {} indicators, maximum is {}",
                analysis.indicator_count, self.max_indicators
            ));
        }

        Ok(())
    }
}

// ============================================================================
// EXAMPLE USAGE
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creative_complex_strategy() {
        let json = r#"
        {
          "condition": {
            "op": "and",
            "conditions": [
              {
                "op": "c_ab",
                "series1": "sma_10",
                "series2": "sma_50"
              },
              {
                "op": "or",
                "conditions": [
                  {
                    "op": "lt",
                    "left": "rsi",
                    "right": 30
                  },
                  {
                    "op": "bet",
                    "value": "macd",
                    "min": -5,
                    "max": 5
                  }
                ]
              }
            ]
          },
          "then": {
            "condition": {
              "op": "gt",
              "left": "volume",
              "right": 1000000
            },
            "then": {
              "action": "BUY",
              "weight": 1.0
            },
            "else": {
              "action": "BUY",
              "weight": 0.5
            }
          },
          "else": "HOLD"
        }
        "#;

        let strategy: StrategyContent = serde_json::from_str(json).unwrap();

        let validator = StrategyValidator::new(
            vec![
                "sma_10".to_string(),
                "sma_50".to_string(),
                "rsi".to_string(),
                "macd".to_string(),
                "volume".to_string(),
            ],
            vec!["BUY".to_string(), "SELL".to_string(), "HOLD".to_string()],
        );

        let analysis = validator.validate_and_analyze(&strategy).unwrap();
        
        println!("Cost Analysis:");
        println!("  Total Cost: {}", analysis.total_cost);
        println!("  Node Count: {}", analysis.node_count);
        println!("  Max Depth: {}", analysis.max_depth);
        println!("  Indicators: {}", analysis.indicator_count);
        println!("  Breakdown: {:?}", analysis.breakdown);

        // Check against budget
        let budget = UserBudget::premium_tier();
        budget.check_budget(&analysis).unwrap();
    }

    #[test]
    fn test_budget_enforcement() {
        // Create a very complex strategy
        let json = r#"
        {
          "condition": {"op": "gt", "left": "sma_10", "right": "sma_50"},
          "then": {
            "condition": {"op": "gt", "left": "sma_20", "right": "sma_100"},
            "then": {
              "condition": {"op": "gt", "left": "sma_30", "right": "sma_200"},
              "then": "BUY",
              "else": "HOLD"
            },
            "else": "SELL"
          },
          "else": "HOLD"
        }
        "#;

        let strategy: StrategyContent = serde_json::from_str(json).unwrap();
        let validator = StrategyValidator::new(
            vec![
                "sma_10".to_string(), "sma_20".to_string(), "sma_30".to_string(),
                "sma_50".to_string(), "sma_100".to_string(), "sma_200".to_string(),
            ],
            vec!["BUY".to_string(), "SELL".to_string(), "HOLD".to_string()],
        );

        let analysis = validator.validate_and_analyze(&strategy).unwrap();

        // Free tier should reject this
        let free_budget = UserBudget::free_tier();
        assert!(free_budget.check_budget(&analysis).is_err());

        // Premium tier should accept it
        let premium_budget = UserBudget::premium_tier();
        assert!(premium_budget.check_budget(&analysis).is_ok());
    }
}
