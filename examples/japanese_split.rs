//! Example: split a Japanese sentence with the bundled model.

use budouy::model::load_default_japanese_parser;

fn main() {
    let parser = load_default_japanese_parser();
    let sentence = "Google の使命は、世界中の情報を整理し、世界中の人がアクセスできて使えるようにすることです。";
    let chunks = parser.parse(sentence);

    println!("{sentence}");
    println!("{chunks:?}");
}
