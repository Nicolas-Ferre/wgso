use crate::RuleError;
use regex::Regex;
use serde::Deserialize;
use serde_valid::Validate;

/// Load rules from YAML bytes.
///
/// The YAML should contain at least a `main` field containing a list of [`Rule`]s.
/// YAML anchors can be used to reuse a sequence of token.
///
/// # Errors
///
/// An error is returned if the parsing of the rules has failed.
///
/// # Examples
///
/// ```yaml
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases_valid/complex/rules.yaml"))]
/// ```
///
/// The previous configuration can be used to parse the following code:
/// ```text
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases_valid/complex/code"))]
/// ```
pub fn load_rules(yaml: &[u8]) -> Result<Vec<Rule>, RuleError> {
    let rules: MainRule = serde_yml::from_slice(yaml).map_err(RuleError::Deserialization)?;
    rules.validate().map_err(RuleError::Validation)?;
    Ok(rules.main)
}

#[derive(Debug, Deserialize, Validate)]
struct MainRule {
    #[validate]
    #[validate(min_items = 1)]
    main: Vec<Rule>,
}

/// A parsing rule.
#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// A single fixed token.
    Token(String),
    /// A single token based on a pattern.
    Pattern(PatternRule),
    /// A repeated group of tokens.
    Repeat(RepeatRule),
    /// A group of tokens chosen according to the next token.
    Choice(
        #[validate]
        #[validate(min_items = 2)]
        Vec<ChoiceRule>,
    ),
}

/// A parsing rule representing a single token based on a pattern.
#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct PatternRule {
    /// A label attached to the extracted token for easier identification.
    #[serde(default)]
    pub label: String,
    /// The token pattern configuration.
    #[validate]
    pub config: PatternConfig,
}

/// A token pattern configuration.
#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct PatternConfig {
    /// A label used for error messages.
    pub label: String,
    /// The regex describing the token.
    #[serde(with = "serde_regex")]
    pub regex: Regex,
    /// The minimum value of the parsed integer.
    ///
    /// If this field is specified, the token must be a valid `i128` value.
    pub min: Option<i128>,
    /// If specified, the maximum value of the parsed integer.
    ///
    /// If this field is specified, the token must be a valid `i128` value.
    pub max: Option<i128>,
}

/// A parsing rule representing a repeated sequence of tokens.
#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
pub struct RepeatRule {
    /// The minimum number of repetitions.
    ///
    /// `0` by default.
    #[serde(default)]
    pub min: u8,
    /// The maximum number of repetitions.
    ///
    /// Infinite by default.
    pub max: Option<u8>,
    /// The group of tokens to repeat.
    #[validate]
    #[validate(min_items = 1)]
    pub group: Vec<Rule>,
}

/// A parsing rule representing a group of tokens only when it starts with a specific token.
#[non_exhaustive]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ChoiceRule {
    /// The starting token.
    pub token: String,
    /// The next tokens.
    #[validate]
    #[validate(min_items = 1)]
    pub next: Vec<Rule>,
}
