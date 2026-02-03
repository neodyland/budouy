# budouy

[![License](https://img.shields.io/crates/l/budouy)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/budouy)](https://crates.io/crates/budouy)
[![Docs.rs](https://img.shields.io/docsrs/budouy)](https://docs.rs/budouy)
[![CI](https://github.com/neodyland/budouy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/neodyland/budouy/actions/workflows/ci.yml)

Rust port of `BudouX` with optional HTML processing and a small CLI.

## Features

- `std`: default feature for std-enabled builds.
- `alloc`: no_std-compatible build using alloc and hashbrown.
- `vendored-models`: bundles default Japanese, Simplified Chinese, Traditional Chinese, and Thai models.
- `html`: enables HTML processing utilities based on `kuchikikiki` (requires `std`).
- `cli`: enables the `budouy` CLI (requires `std`, implies `vendored-models`).

Note: `std` and `alloc` are mutually exclusive.

## Usage

### Library

Custom model:

```rust
use std::collections::HashMap;
use budouy::{Model, Parser};
use budouy::model::FeatureKey;

let mut model: Model = HashMap::new();
model.insert(FeatureKey::UW4, HashMap::from([("a".to_string(), 10_000)]));

let parser = Parser::new(model);
let chunks = parser.parse("abcdeabcd");
assert_eq!(chunks, vec!["abcde", "abcd"]);
```

Default model (requires `vendored-models`):

```rust
use budouy::model::load_default_japanese_parser;

let parser = load_default_japanese_parser();
let chunks = parser.parse("今日は良い天気です");
println!("{:?}", chunks);
```

HTML processing (requires `html` + `vendored-models`):

```rust
use budouy::HTMLProcessingParser;
use budouy::model::load_default_japanese_parser;

let parser = load_default_japanese_parser();
let html_parser = HTMLProcessingParser::new(parser, None);
let input = "今日は<strong>良い</strong>天気です";
let output = html_parser.translate_html_string(input);
println!("{}", output);
```

### CLI

Build and run the CLI (requires `cli`):

```bash
cargo run --features cli -- parse --lang ja "今日は良い天気です"
```

Use a custom model JSON:

```bash
cargo run --features cli -- parse --model ./model.json "今日は良い天気です"
```

Read from stdin:

```bash
echo "今日は良い天気です" | cargo run --features cli -- parse --lang ja
```

## no_std

This crate supports `no_std` with `alloc`. Disable default features and enable `alloc`:

```toml
budouy = { version = "0.1", default-features = false, features = ["alloc"] }
```

`std` and `alloc` are mutually exclusive. The `html` and `cli` features require `std`.

## Models

Vendored models in `src/models/*.json` are derived from the original BudouX
project (Google) and are licensed under Apache-2.0. See `LICENSE` for details.
This project is not affiliated with Google.

## License

Apache-2.0. See `LICENSE`.
