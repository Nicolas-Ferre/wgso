use crate::RuleError;
use serde::Deserialize;
use serde_valid::Validate;

pub fn load_rules(yaml: &[u8]) -> Result<Vec<Rule>, RuleError> {
    let rules: MainRule = serde_yml::from_slice(yaml).map_err(RuleError::Deserialization)?;
    rules.validate().map_err(RuleError::Validation)?;
    Ok(rules.main)
}

#[derive(Debug, Deserialize, Validate)]
struct MainRule {
    #[validate]
    main: Vec<Rule>,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    Token(String),
    Pattern(PatternRule),
    Repeat(RepeatRule),
    Choice(
        #[validate]
        #[validate(min_items = 2)]
        Vec<ChoiceRule>,
    ),
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct PatternRule {
    #[serde(default)]
    pub label: String,
    #[validate]
    pub config: PatternConfig,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct PatternConfig {
    pub label: String,
    pub min: Option<i128>,
    pub max: Option<i128>,
    pub is_digit_prefix_allowed: Option<bool>,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct RepeatRule {
    #[serde(default)]
    pub min: u8,
    pub max: Option<u8>,
    #[validate]
    #[validate(min_items = 1)]
    pub group: Vec<Rule>,
}

#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ChoiceRule {
    pub token: String,
    #[validate]
    pub next: Vec<Rule>,
}
