//! `BudouX` parser port in Rust.
//!
//! This crate provides the core `BudouX` segmentation algorithm and optional
//! HTML processing utilities. The parser splits a sentence into semantic
//! chunks based on a trained model.
//!
//! # Features
//! - `std`: Default feature for std-enabled builds.
//! - `alloc`: `no_std`-compatible build using `alloc` and `hashbrown`.
//! - `vendored-models`: Bundles default Japanese/Chinese/Thai models.
//! - `html`: Enables HTML processing utilities based on `kuchikikiki` (requires `std`).
//! - `cli`: Enables the `budouy` CLI (requires `std`, implies `vendored-models`).
//! - `wasm`: Enables WebAssembly bindings via `wasm-bindgen` (implies `alloc` and `vendored-models`).
//!
//! Note: `std` and `alloc` are mutually exclusive.
//!
//! # `no_std`
//! This crate supports `no_std` with `alloc`. Disable default features and enable `alloc`:
//! ```toml
//! budouy = { version = "0.1", default-features = false, features = ["alloc"] }
//! ```
//! The `html` and `cli` features require `std`.
//!
//! # Examples
//!
//! Parse a sentence with a custom model:
//! ```rust
//! use std::collections::HashMap;
//! use budouy::{Model, Parser};
//! use budouy::model::FeatureKey;
//!
//! let mut model: Model = HashMap::new();
//! model.insert(FeatureKey::UW4, HashMap::from([("a".to_string(), 10_000)]));
//! let parser = Parser::new(model);
//! let chunks = parser.parse("abcdeabcd");
//! assert_eq!(chunks, vec!["abcde", "abcd"]);
//! ```
//!
//! Use the default Japanese model (requires `vendored-models`):
//! ```rust,no_run
//! use budouy::model::load_default_japanese_parser;
//!
//! let parser = load_default_japanese_parser();
//! let chunks = parser.parse("今日は良い天気です");
//! println!("{:?}", chunks);
//! ```
//!
//! Process HTML (requires `html` + `vendored-models`):
//! ```rust,no_run
//! use budouy::{HTMLProcessingParser, model::load_default_japanese_parser};
//!
//! let parser = load_default_japanese_parser();
//! let html_parser = HTMLProcessingParser::new(parser, None);
//! let input = "今日は<strong>良い</strong>天気です";
//! let output = html_parser.translate_html_string(input);
//! println!("{}", output);
//! ```
//!
//! # WebAssembly
//!
//! Build for web with `wasm-pack`:
//! ```bash
//! wasm-pack build --target web --no-default-features --features wasm
//! ```
//!
//! Use from JavaScript:
//! ```javascript
//! import init, { BudouY } from './pkg/budouy.js';
//!
//! await init();
//! const parser = BudouY.japanese();
//! const chunks = parser.parse("今日は良い天気です");
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Enable only one of the `std` or `alloc` features.");

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
compile_error!("Either the `std` or `alloc` feature must be enabled.");

pub(crate) mod map {
    #[cfg(feature = "std")]
    #[expect(clippy::absolute_paths)]
    pub type HashMap<K, V> = std::collections::HashMap<K, V>;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub type HashMap<K, V> = hashbrown::HashMap<K, V>;
}

/// Model types and loaders.
pub mod model;
mod parser;

#[cfg(feature = "html")]
mod html_processor;

#[cfg(feature = "wasm")]
mod wasm;

#[doc(inline)]
pub use model::Model;
pub use parser::Parser;

#[cfg(feature = "html")]
pub use html_processor::{HTMLProcessingParser, HTMLProcessor, HTMLProcessorOptions, Separator};
