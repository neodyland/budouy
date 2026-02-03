use alloc::string::String;
use alloc::vec::Vec;

use crate::Model;
use crate::model::FeatureKey;

/// `BudouX` parser for semantic line breaks.
#[derive(Clone, Debug)]
pub struct Parser {
    model: Model,
    base_score: f64,
}

impl Parser {
    /// Create a new parser from a model.
    #[must_use]
    pub fn new(model: Model) -> Self {
        let total: f64 = model
            .values()
            .flat_map(|group| group.values())
            .map(|value| f64::from(*value))
            .sum();
        let base_score = -0.5 * total;
        Self { model, base_score }
    }

    /// Split a sentence into semantic chunks.
    #[must_use]
    pub fn parse(&self, sentence: &str) -> Vec<String> {
        if sentence.is_empty() {
            return Vec::new();
        }
        let chars: Vec<char> = sentence.chars().collect();
        let boundaries = self.parse_boundaries_from_chars(&chars);
        split_by_boundaries(&chars, &boundaries)
    }

    /// Return the boundary indices for the sentence.
    ///
    /// Indices are based on `char` positions.
    #[must_use]
    pub fn parse_boundaries(&self, sentence: &str) -> Vec<usize> {
        let chars: Vec<char> = sentence.chars().collect();
        self.parse_boundaries_from_chars(&chars)
    }

    fn parse_boundaries_from_chars(&self, chars: &[char]) -> Vec<usize> {
        let mut result = Vec::new();
        let len = chars.len();
        if len == 0 {
            return result;
        }

        for i in 1..len {
            let mut score = self.base_score;
            score += self.weight(
                FeatureKey::UW1,
                &substring(chars, i.saturating_sub(3), i.saturating_sub(2)),
            );
            score += self.weight(
                FeatureKey::UW2,
                &substring(chars, i.saturating_sub(2), i.saturating_sub(1)),
            );
            score += self.weight(FeatureKey::UW3, &substring(chars, i.saturating_sub(1), i));
            score += self.weight(FeatureKey::UW4, &substring(chars, i, (i + 1).min(len)));
            score += self.weight(
                FeatureKey::UW5,
                &substring(chars, (i + 1).min(len), (i + 2).min(len)),
            );
            score += self.weight(
                FeatureKey::UW6,
                &substring(chars, (i + 2).min(len), (i + 3).min(len)),
            );
            score += self.weight(FeatureKey::BW1, &substring(chars, i.saturating_sub(2), i));
            score += self.weight(
                FeatureKey::BW2,
                &substring(chars, i.saturating_sub(1), (i + 1).min(len)),
            );
            score += self.weight(FeatureKey::BW3, &substring(chars, i, (i + 2).min(len)));
            score += self.weight(FeatureKey::TW1, &substring(chars, i.saturating_sub(3), i));
            score += self.weight(
                FeatureKey::TW2,
                &substring(chars, i.saturating_sub(2), (i + 1).min(len)),
            );
            score += self.weight(
                FeatureKey::TW3,
                &substring(chars, i.saturating_sub(1), (i + 2).min(len)),
            );
            score += self.weight(FeatureKey::TW4, &substring(chars, i, (i + 3).min(len)));
            if score > 0.0 {
                result.push(i);
            }
        }
        result
    }

    fn weight(&self, group: FeatureKey, key: &str) -> f64 {
        f64::from(
            self.model
                .get(&group)
                .and_then(|map| map.get(key))
                .copied()
                .unwrap_or(0),
        )
    }
}

fn substring(chars: &[char], start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    chars[start..end].iter().collect()
}

fn split_by_boundaries(chars: &[char], boundaries: &[usize]) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;
    for &boundary in boundaries {
        result.push(chars[start..boundary].iter().collect());
        start = boundary;
    }
    result.push(chars[start..].iter().collect());
    result
}
