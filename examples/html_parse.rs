//! Example: apply `BudouX` boundaries to HTML content.

use budouy::HTMLProcessingParser;
use budouy::model::load_default_japanese_parser;

const ZWSP: &str = "\u{200B}";

fn main() {
    let parser = load_default_japanese_parser();
    let html_parser = HTMLProcessingParser::new(parser, None);

    let input = "html 対応バージョンの Google の使命は、世界中の情報を<strong>整理</strong>し、<em>世界中の人がアクセス</em>できて使えるようにすることです。";
    let output = html_parser.translate_html_string(input);

    println!("{input}");
    println!("{}", output.replace(ZWSP, "\x1b[96m<ZWSP>\x1b[0m"));
}
