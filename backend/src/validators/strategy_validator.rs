use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub fn validate_strategy_title(title: &str) -> Result<(), AppError> {
    if title.is_empty() {
        return Err(AppError::BadRequest("title is required".to_string()));
    }

    if title.len() > 100 {
        return Err(AppError::BadRequest(
            "Title cannot be longer than 100 characters".to_string(),
        ));
    }

    let chars: Vec<char> = title.chars().collect(); // TODO: This collect might be bad

    if !chars
        .iter()
        .all(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
    {
        return Err(AppError::BadRequest(
            "Must be a valid title".to_string(),
        ));
    }


    if chars.first() == Some(&'-') || chars.first() == Some(&'_') {
        return Err(AppError::BadRequest(
            "Must be a valid title".to_string(),
        ));
    }

    if chars.last() == Some(&'-') || chars.last() == Some(&'_') {
        return Err(AppError::BadRequest(
            "Must be a valid title".to_string(),
        ));
    }

    Ok(())
}


#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid strategy type: {0}")]
    InvalidStrategyType(String),
    #[error("Invalid action type '{0}' for strategy type '{1}'")]
    InvalidActionType(String, String),
    #[error("Weight must be between 0 and 1, got {0}")]
    InvalidWeight(f64),
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Actions array cannot be empty")]
    EmptyActions,
    #[error("Invalid condition structure: {0}")]
    InvalidCondition(String),
    #[error("MessagePack serialization error: {0}")]
    SerializationError(String),
    #[error("MessagePack deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid indicator: {0}")]
    InvalidIndicator(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StrategyType {
    Spot,
    Options,
}

impl StrategyType {
    fn valid_actions(&self) -> HashSet<&'static str> {
        match self {
            StrategyType::Spot => {
                let mut set = HashSet::new();
                set.insert("buy");
                set.insert("sell");
                set
            }
            StrategyType::Options => {
                let mut set = HashSet::new();
                set.insert("long");
                set.insert("short");
                set
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "type")]
    pub strategy_type: StrategyType,
    //#[serde(flatten)]
    //pub extra: std::collections::HashMap<String, rmpv::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Number(f64),
    //Boolean(bool),
    Indicator(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Cond {
    And { conds: Vec<Cond> },
    Or { conds: Vec<Cond> },
    Not { cond: Box<Cond> },
    #[serde(rename = "lt")]
    LessThan { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "gt")]
    GreaterThan { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "le")]
    LessThanOrEqual { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "ge")]
    GreaterThanOrEqual { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "eq")]
    Equal { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "neq")]
    NotEqual { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "bet")]
    Between { val: Box<Value>, min: Box<Value>, max: Box<Value> },
    #[serde(rename = "xab")]
    CrossesAbove { l: Box<Value>, r: Box<Value> },
    #[serde(rename = "xbe")]
    CrossesBelow { l: Box<Value>, r: Box<Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub w: f64,
    pub cond: Cond,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyContent {
    pub meta: Meta,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub struct StrategyValidator {
    valid_indicators: HashSet<String>,
}

impl StrategyValidator {
    pub fn new(valid_indicators: HashSet<String>) -> Self {
        Self {
            valid_indicators: valid_indicators,
        }
    }

    pub fn update_valid_indicators(&mut self, new_indicators: HashSet<String>) -> () {
        self.valid_indicators = new_indicators;
    }

    pub fn get_indicators(strategy: &StrategyContent) -> HashSet<String> {
        let mut indicators = HashSet::new();

        for action in &strategy.actions {
            Self::collect_indicators_from_condition(&action.cond, &mut indicators);
        }

        indicators
    }

    fn collect_indicators_from_condition(condition: &Cond, indicators: &mut HashSet<String>) {
        match condition {
            Cond::GreaterThan { l, r }
            | Cond::LessThan { l, r }
            | Cond::GreaterThanOrEqual { l, r }
            | Cond::LessThanOrEqual { l, r }
            | Cond::Equal { l, r }
            | Cond::NotEqual { l, r }
            | Cond::CrossesAbove { l, r }
            | Cond::CrossesBelow { l, r }
            => {
                Self::collect_indicators_from_value(l, indicators);
                Self::collect_indicators_from_value(r, indicators);
            }
            Cond::And { conds } | Cond::Or { conds } => {
                if conds.is_empty() {
                    return ;
                }
                for cond in conds {
                    Self::collect_indicators_from_condition(cond, indicators);
                }
            }
            Cond::Not { cond } => {
                Self::collect_indicators_from_condition(cond, indicators);
            }
            Cond::Between { val, min, max } => {
                Self::collect_indicators_from_value(val, indicators);
                Self::collect_indicators_from_value(min, indicators);
                Self::collect_indicators_from_value(max, indicators);
            }
        }
    }

    fn collect_indicators_from_value(val: &Value, indicators: &mut HashSet<String>) {
        if let Value::Indicator(ta) = val {
            indicators.insert(ta.to_string());
        }
    }

    /// Validate and deserialize from MessagePack bytes
    pub fn validate_msgpack(&self, bytes: &[u8]) -> Result<StrategyContent, ValidationError> {
        let strategy: StrategyContent = rmp_serde::from_slice(bytes)
            .map_err(|e| ValidationError::DeserializationError(e.to_string()))?;

        self.validate_strategy(&strategy)?;
        Ok(strategy)
    }

    /// Validate and serialize to MessagePack bytes
    pub fn to_msgpack(&self, strategy: &StrategyContent) -> Result<Vec<u8>, ValidationError> {
        self.validate_strategy(strategy)?;
        
        rmp_serde::to_vec(strategy)
            .map_err(|e| ValidationError::SerializationError(e.to_string()))
    }

    /// Validate from JSON (for user input/debugging)
    pub fn validate_json(&self, json_str: &str) -> Result<StrategyContent, ValidationError> {
        let strategy: StrategyContent = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::DeserializationError(e.to_string()))?;

        self.validate_strategy(&strategy)?;
        Ok(strategy)
    }

    /// Convert JSON to MessagePack (for user input)
    pub fn json_to_msgpack(&self, json_str: &str) -> Result<Vec<u8>, ValidationError> {
        let strategy = self.validate_json(json_str)?;
        self.to_msgpack(&strategy)
    }

    /// Convert MessagePack to JSON (for debugging)
    pub fn msgpack_to_json(&self, bytes: &[u8]) -> Result<String, ValidationError> {
        let strategy = self.validate_msgpack(bytes)?;
        serde_json::to_string(&strategy)
            .map_err(|e| ValidationError::SerializationError(e.to_string()))
    }

    pub fn validate_strategy(&self, strategy: &StrategyContent) -> Result<(), ValidationError> {
        if strategy.actions.is_empty() {
            return Err(ValidationError::EmptyActions);
        }

        let valid_actions = strategy.meta.strategy_type.valid_actions();

        for action in &strategy.actions {
            self.validate_action(action, &valid_actions, &strategy.meta.strategy_type)?;
        }

        Ok(())
    }

    fn validate_action(
        &self,
        action: &Action,
        valid_actions: &HashSet<&str>,
        strategy_type: &StrategyType,
    ) -> Result<(), ValidationError> {
        if !valid_actions.contains(action.action_type.as_str()) {
            return Err(ValidationError::InvalidActionType(
                action.action_type.clone(),
                format!("{:?}", strategy_type),
            ));
        }

        if action.w < 0.0 || action.w > 1.0 {
            return Err(ValidationError::InvalidWeight(action.w));
        }

        self.validate_condition(&action.cond)?;

        Ok(())
    }

    fn validate_condition(&self, cond: &Cond) -> Result<(), ValidationError> {
        match cond {
            Cond::GreaterThan { l, r }
            | Cond::LessThan { l, r }
            | Cond::GreaterThanOrEqual { l, r }
            | Cond::LessThanOrEqual { l, r }
            | Cond::Equal { l, r }
            | Cond::NotEqual { l, r }
            | Cond::CrossesAbove { l, r }
            | Cond::CrossesBelow { l, r }
            => {
                self.validate_value(l)?;
                self.validate_value(r)?;
            }
            Cond::And { conds } | Cond::Or { conds } => {
                if conds.is_empty() {
                    return Err(ValidationError::MissingField("Empty logical operator".to_string()));
                }
                for cond in conds {
                    self.validate_condition(cond)?;
                }
            }
            Cond::Not { cond } => {
                self.validate_condition(cond)?;
            }
            Cond::Between { val, min, max } => {
                self.validate_value(val)?;
                self.validate_value(min)?;
                self.validate_value(max)?;
            }
        }

        Ok(())
    }

    fn validate_value(&self, val: &Value) -> Result<(), ValidationError> {
        if let Value::Indicator(ta) = val {
            if !self.valid_indicators.contains(ta) {
                return Err(ValidationError::InvalidIndicator(ta.to_string()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_msgpack_conversion() {
        let json = r#"
        {
          "meta": {
            "type": "spot"
          },
          "actions": [
            {
              "type": "buy",
              "w": 0.8,
              "cond": {
                "gt": {
                  "l": "sma_10",
                  "r": "sma_50"
                }
              }
            }
          ]
        }"#;

        let strat_validator = StrategyValidator::new(vec!["sma_10".to_string(), "sma_50".to_string()]);

        let msgpack = strat_validator.json_to_msgpack(json).unwrap();
        let back_to_json = strat_validator.msgpack_to_json(&msgpack).unwrap();
        
        // Verify round-trip works
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let converted: serde_json::Value = serde_json::from_str(&back_to_json).unwrap();
        assert_eq!(original, converted);
    }

    #[test]
    fn test_msgpack_size_savings() {
        let json = r#"
        {
            "meta": {
                "type": "spot"
            },
            "actions": [
            {
                "type": "sell",
                "w": 1.0,
                "cond": {
                    "and": {
                        "conds": [
                        {
                            "lt": {
                                "l": "rsi",
                                "r": 30
                            }
                        },
                        {
                            "gt": {
                                "l": "macd",
                                "r": 0
                            }
                        }
                        ]
                    }
                }
            }
            ]
        }"#;

        let strat_validator = StrategyValidator::new(vec!["rsi".to_string(), "macd".to_string()]);
        let msgpack = strat_validator.json_to_msgpack(json).unwrap();
        let json_size = json.as_bytes().len();
        let msgpack_size = msgpack.len();
        
        println!("JSON size: {} bytes", json_size);
        println!("MessagePack size: {} bytes", msgpack_size);
        println!("Savings: {:.1}%", (1.0 - msgpack_size as f64 / json_size as f64) * 100.0);
        
        assert!(msgpack_size < json_size);
    }

    #[test]
    fn test_valid_msgpack_strategy1() {
        let json = r#"
        {
            "meta": {
                "type": "options"
            },
            "actions": [
            {
                "type": "long",
                "w": 0.5,
                "cond": {
                    "bet": {
                        "val": "iv_rank",
                        "min": 20,
                        "max": 60
                    }
                }
            }
            ]
        }"#;

        let strat_validator = StrategyValidator::new(vec!["iv_rank".to_string()]);
        let msgpack = strat_validator.json_to_msgpack(json).unwrap();
        assert!(strat_validator.validate_msgpack(&msgpack).is_ok());
    }

    #[test]
    fn test_valid_msgpack_strategy2() {
        let json = r#"
        {
            "meta": {
                "type": "options"
            },
            "actions": [
            {
                "type": "short",
                "w": 0.3,
                "cond": {
                    "or": {
                        "conds": [
                        {
                            "xab": {
                                "l": "sma_20",
                                "r": "sma_100"
                            }
                        },
                        {
                            "gt": {
                                "l": "vix",
                                "r": 25
                            }
                        }
                        ]
                    }
                }
            }
            ]
        }"#;

        let strat_validator = StrategyValidator::new(vec!["sma_20".to_string(), "sma_100".to_string(), "vix".to_string()]);
        let msgpack = strat_validator.json_to_msgpack(json).unwrap();
        assert!(strat_validator.validate_msgpack(&msgpack).is_ok());
    }

    #[test]
    fn test_valid_msgpack_strategy3() {
        let json = r#"
        {
            "meta": {
                "type": "spot"
            },
            "actions": [
            {
                "type": "buy",
                "w": 0.9,
                "cond": {
                    "not": {
                        "cond": {
                            "and": {
                                "conds": [
                                {
                                    "le": {
                                        "l": "rsi",
                                        "r": 30
                                    }
                                },
                                {
                                    "gt": {
                                        "l": "volume",
                                        "r": 1000000 
                                    }
                                }
                                ]
                            }
                        }
                    }
                }
            }
            ]
        }"#;

        let strat_validator = StrategyValidator::new(vec!["rsi".to_string(), "volume".to_string()]);
        let msgpack = strat_validator.json_to_msgpack(json).unwrap();
        assert!(strat_validator.validate_msgpack(&msgpack).is_ok());
    }


    #[test]
    fn test_invalid_action_for_spot() {
        let json = r#"{ "meta": { "type": "spot" }, "actions": [{ "type": "long", "w": 0.5, "cond": { "eq": { "l": 1, "r": 1 } } }] }"#;

        let strat_val = StrategyValidator::new(vec![]);
        let result = strat_val.json_to_msgpack(json);
        assert!(matches!(result, Err(ValidationError::InvalidActionType(_, _))));
    }

    #[test]
    fn test_invalid_weight() {
        let json = r#"{ "meta": { "type": "options" }, "actions": [{ "type": "short", "w": 2.0, "cond": { "gt": { "l": 1, "r": 0 } } }] }"#;

        let strat_val = StrategyValidator::new(vec![]);
        let result = strat_val.json_to_msgpack(json);
        assert!(matches!(result, Err(ValidationError::InvalidWeight(_))));
    }

    #[test]
    fn test_empty_logical_op() {
        let json = r#"{ "meta": { "type": "spot" }, "actions": [{ "type": "buy", "w": 0.5, "cond": { "and": { "conds": [] } } }] }"#;

        let strat_val = StrategyValidator::new(vec![]);
        let result = strat_val.json_to_msgpack(json);
        assert!(matches!(result, Err(ValidationError::MissingField(_))));
    }

    #[test]
    fn test_invalid_indicator() {
        let json = r#"
        {
          "meta": {
            "type": "spot"
          },
          "actions": [
            {
              "type": "buy",
              "w": 0.8,
              "cond": {
                "gt": {
                  "l": "sma_10",
                  "r": "sma_50"
                }
              }
            }
          ]
        }"#;

        let strat_val = StrategyValidator::new(vec![]);
        let result = strat_val.json_to_msgpack(json);
        assert!(matches!(result, Err(ValidationError::InvalidIndicator(_))));
    }
}


/*

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


*/
