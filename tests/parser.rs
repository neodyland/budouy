//! Parser tests.
#![allow(missing_docs)]

use budouy::Parser;
use budouy::model::FeatureKey;
use std::collections::HashMap;

#[test]
fn parse_separates_on_strong_feature() {
    let mut model: HashMap<FeatureKey, HashMap<String, i32>> = HashMap::new();
    model.insert(FeatureKey::UW4, HashMap::from([("a".to_string(), 10_000)]));
    let parser = Parser::new(model);
    let result = parser.parse("abcdeabcd");
    assert_eq!(result, vec!["abcde", "abcd"]);
}

#[test]
fn parse_separates_even_single_char_phrase() {
    let mut model: HashMap<FeatureKey, HashMap<String, i32>> = HashMap::new();
    model.insert(FeatureKey::UW4, HashMap::from([("b".to_string(), 10_000)]));
    let parser = Parser::new(model);
    let result = parser.parse("abcdeabcd");
    assert_eq!(result, vec!["a", "bcdea", "bcd"]);
}

#[test]
fn parse_empty_returns_empty_vec() {
    let parser = Parser::new(HashMap::new());
    let result = parser.parse("");
    assert!(result.is_empty());
}
