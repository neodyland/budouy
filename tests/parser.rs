//! Parser tests.

use budouy::model::{FeatureKey, InnerModel};
use budouy::{Model, Parser};

#[test]
fn parse_separates_on_strong_feature() {
    let mut model: Model = Model::new();
    let mut inner = InnerModel::new();
    inner.insert("a".to_string(), 10_000);
    model.insert(FeatureKey::UW4, inner);
    let parser = Parser::new(model);
    let result = parser.parse("abcdeabcd");
    assert_eq!(result, vec!["abcde", "abcd"]);
}

#[test]
fn parse_separates_even_single_char_phrase() {
    let mut model: Model = Model::new();
    let mut inner = InnerModel::new();
    inner.insert("b".to_string(), 10_000);
    model.insert(FeatureKey::UW4, inner);
    let parser = Parser::new(model);
    let result = parser.parse("abcdeabcd");
    assert_eq!(result, vec!["a", "bcdea", "bcd"]);
}

#[test]
fn parse_empty_returns_empty_vec() {
    let parser = Parser::new(Model::new());
    let result = parser.parse("");
    assert!(result.is_empty());
}
