//! Model types and loaders.

use alloc::string::String;
use core::fmt;
use core::str::FromStr;

use crate::map::HashMap;

use thiserror::Error;

/// Feature keys used by the `BudouX` model.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FeatureKey {
    /// Unigram feature at offset -3.
    UW1,
    /// Unigram feature at offset -2.
    UW2,
    /// Unigram feature at offset -1.
    UW3,
    /// Unigram feature at offset 0.
    UW4,
    /// Unigram feature at offset +1.
    UW5,
    /// Unigram feature at offset +2.
    UW6,
    /// Bigram feature at offset -2..0.
    BW1,
    /// Bigram feature at offset -1..+1.
    BW2,
    /// Bigram feature at offset 0..+2.
    BW3,
    /// Trigram feature at offset -3..0.
    TW1,
    /// Trigram feature at offset -2..+1.
    TW2,
    /// Trigram feature at offset -1..+2.
    TW3,
    /// Trigram feature at offset 0..+3.
    TW4,
}

impl FeatureKey {
    /// Return the canonical string for this feature key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UW1 => "UW1",
            Self::UW2 => "UW2",
            Self::UW3 => "UW3",
            Self::UW4 => "UW4",
            Self::UW5 => "UW5",
            Self::UW6 => "UW6",
            Self::BW1 => "BW1",
            Self::BW2 => "BW2",
            Self::BW3 => "BW3",
            Self::TW1 => "TW1",
            Self::TW2 => "TW2",
            Self::TW3 => "TW3",
            Self::TW4 => "TW4",
        }
    }
}

impl FromStr for FeatureKey {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "UW1" => Ok(Self::UW1),
            "UW2" => Ok(Self::UW2),
            "UW3" => Ok(Self::UW3),
            "UW4" => Ok(Self::UW4),
            "UW5" => Ok(Self::UW5),
            "UW6" => Ok(Self::UW6),
            "BW1" => Ok(Self::BW1),
            "BW2" => Ok(Self::BW2),
            "BW3" => Ok(Self::BW3),
            "TW1" => Ok(Self::TW1),
            "TW2" => Ok(Self::TW2),
            "TW3" => Ok(Self::TW3),
            "TW4" => Ok(Self::TW4),
            _ => Err(()),
        }
    }
}

impl fmt::Display for FeatureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Errors that can occur when parsing a model JSON string.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ModelError {
    /// The JSON input is invalid.
    #[error("invalid model json: {0}")]
    Json(#[from] serde_json::Error),
    /// A feature key was not recognized.
    #[error("unknown feature key: {0}")]
    UnknownFeature(String),
}

/// `BudouX` model data.
///
/// The outer map is keyed by feature names (e.g., `UW1`, `BW2`), and the inner
/// map contains string features and their weights.
pub type Model = HashMap<FeatureKey, InnerModel>;

/// Inner model map containing string features and their weights.
pub type InnerModel = HashMap<String, i32>;

/// Parse a JSON string into a [`Model`].
///
/// The JSON structure must match the `BudouX` model format.
/// Unknown feature keys return [`ModelError::UnknownFeature`].
///
/// # Errors
/// - Returns [`ModelError::Json`] if the input is not valid JSON.
/// - Returns [`ModelError::UnknownFeature`] if a feature key is not supported.
pub fn parse_model_json(input: &str) -> Result<Model, ModelError> {
    let raw: HashMap<String, HashMap<String, i32>> = serde_json::from_str(input)?;
    let mut model = HashMap::with_capacity(raw.len());
    for (key, value) in raw {
        let parsed = key
            .parse::<FeatureKey>()
            .map_err(|()| ModelError::UnknownFeature(key))?;
        model.insert(parsed, value);
    }
    Ok(model)
}

#[cfg(feature = "vendored-models")]
mod vendored {
    use super::{Model, parse_model_json};
    use crate::Parser;
    use crate::map::HashMap;
    #[cfg(feature = "std")]
    use std::sync::LazyLock;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use spin::Lazy as SpinLazy;

    #[cfg(feature = "std")]
    type Lazy<T> = LazyLock<T>;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    type Lazy<T> = SpinLazy<T>;

    static JA_MODEL: Lazy<Model> = Lazy::new(|| {
        parse_model_json(include_str!("models/ja.json")).expect("invalid ja model json")
    });
    static ZH_HANS_MODEL: Lazy<Model> = Lazy::new(|| {
        parse_model_json(include_str!("models/zh-hans.json")).expect("invalid zh-hans model json")
    });
    static ZH_HANT_MODEL: Lazy<Model> = Lazy::new(|| {
        parse_model_json(include_str!("models/zh-hant.json")).expect("invalid zh-hant model json")
    });
    static TH_MODEL: Lazy<Model> = Lazy::new(|| {
        parse_model_json(include_str!("models/th.json")).expect("invalid th model json")
    });

    /// Load the default Japanese model parser.
    pub fn load_default_japanese_parser() -> Parser {
        Parser::new(JA_MODEL.clone())
    }

    /// Load the default Simplified Chinese model parser.
    pub fn load_default_simplified_chinese_parser() -> Parser {
        Parser::new(ZH_HANS_MODEL.clone())
    }

    /// Load the default Traditional Chinese model parser.
    pub fn load_default_traditional_chinese_parser() -> Parser {
        Parser::new(ZH_HANT_MODEL.clone())
    }

    /// Load the default Thai model parser.
    pub fn load_default_thai_parser() -> Parser {
        Parser::new(TH_MODEL.clone())
    }

    #[must_use]
    /// Load all available default parsers keyed by language code.
    pub fn load_default_parsers() -> HashMap<&'static str, Parser> {
        HashMap::from([
            ("ja", load_default_japanese_parser()),
            ("zh-hans", load_default_simplified_chinese_parser()),
            ("zh-hant", load_default_traditional_chinese_parser()),
            ("th", load_default_thai_parser()),
        ])
    }
}

#[cfg(feature = "vendored-models")]
pub use vendored::{
    load_default_japanese_parser, load_default_parsers, load_default_simplified_chinese_parser,
    load_default_thai_parser, load_default_traditional_chinese_parser,
};
