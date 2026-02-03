//! WebAssembly bindings for `BudouY`.
//!
//! This module provides JavaScript-friendly wrappers around the core parser.

use alloc::string::String;
use alloc::vec::Vec;

use wasm_bindgen::prelude::*;

use crate::Parser;
use crate::model::{
    load_default_japanese_parser, load_default_simplified_chinese_parser, load_default_thai_parser,
    load_default_traditional_chinese_parser,
};

/// `BudouY` parser for JavaScript.
///
/// Use the static methods to create a parser for a specific language,
/// then call `parse()` to split text into semantic chunks.
#[derive(Debug)]
#[wasm_bindgen(js_name = BudouY)]
pub struct JsParser {
    inner: Parser,
}

#[wasm_bindgen(js_class = BudouY)]
impl JsParser {
    /// Create a parser for Japanese text.
    #[wasm_bindgen(js_name = japanese)]
    pub fn japanese() -> Self {
        Self {
            inner: load_default_japanese_parser(),
        }
    }

    /// Create a parser for Simplified Chinese text.
    #[wasm_bindgen(js_name = simplifiedChinese)]
    pub fn simplified_chinese() -> Self {
        Self {
            inner: load_default_simplified_chinese_parser(),
        }
    }

    /// Create a parser for Traditional Chinese text.
    #[wasm_bindgen(js_name = traditionalChinese)]
    pub fn traditional_chinese() -> Self {
        Self {
            inner: load_default_traditional_chinese_parser(),
        }
    }

    /// Create a parser for Thai text.
    #[wasm_bindgen(js_name = thai)]
    pub fn thai() -> Self {
        Self {
            inner: load_default_thai_parser(),
        }
    }

    /// Split a sentence into semantic chunks.
    ///
    /// Returns an array of strings representing the chunks.
    #[wasm_bindgen]
    pub fn parse(&self, sentence: &str) -> Vec<String> {
        self.inner.parse(sentence)
    }

    /// Return the boundary indices for the sentence.
    ///
    /// Indices are based on Unicode code point positions.
    #[wasm_bindgen(js_name = parseBoundaries)]
    pub fn parse_boundaries(&self, sentence: &str) -> Vec<usize> {
        self.inner.parse_boundaries(sentence)
    }
}
