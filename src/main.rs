//! `BudouY` CLI entrypoint.

use std::io::{self, Read};
use std::{env, fs};

use seahorse::{App, Command, Context, Flag, FlagType};

use budouy::Parser;
use budouy::model::{
    load_default_japanese_parser, load_default_parsers, load_default_simplified_chinese_parser,
    load_default_thai_parser, load_default_traditional_chinese_parser, parse_model_json,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let app = App::new("budouy")
        .description("BudouX parser CLI")
        .usage("budouy <command> [options]")
        .command(parse_command())
        .action(|c| {
            if c.args.is_empty() {
                eprintln!("No command specified. Use 'budouy parse --help'.");
            }
        });

    app.run(args);
}

fn parse_command() -> Command {
    Command::new("parse")
        .description("Parse a sentence using a model JSON file or a default model")
        .usage("budouy parse --model MODEL.json | --lang LANG [--separator SEP] [TEXT]")
        .flag(Flag::new("model", FlagType::String).description("Path to model JSON"))
        .flag(Flag::new("lang", FlagType::String).description("Default model language code"))
        .flag(
            Flag::new("separator", FlagType::String).description("Chunk separator (default: '|')"),
        )
        .action(parse_action)
}

fn parse_action(c: &Context) {
    let model_path = c.string_flag("model").ok();
    let lang = c.string_flag("lang").ok();

    if model_path.is_some() && lang.is_some() {
        eprintln!("Specify either --model or --lang, not both.");
        return;
    }
    if model_path.is_none() && lang.is_none() {
        eprintln!("Missing --model or --lang.");
        eprintln!("Available --lang values: ja, zh-hans, zh-hant, th");
        return;
    }

    let separator = c
        .string_flag("separator")
        .unwrap_or_else(|_| "|".to_string());

    let input = if c.args.is_empty() {
        read_stdin().unwrap_or_default()
    } else {
        c.args.join(" ")
    };

    let parser = if let Some(path) = model_path {
        let model_json = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Failed to read model file: {err}");
                return;
            }
        };
        let model = match parse_model_json(&model_json) {
            Ok(model) => model,
            Err(err) => {
                eprintln!("Failed to parse model: {err}");
                return;
            }
        };
        Parser::new(model)
    } else {
        match lang.as_deref() {
            Some("ja") => load_default_japanese_parser(),
            Some("zh-hans") => load_default_simplified_chinese_parser(),
            Some("zh-hant") => load_default_traditional_chinese_parser(),
            Some("th") => load_default_thai_parser(),
            Some(code) => {
                eprintln!("Unknown --lang value: {code}");
                eprintln!("Available --lang values: ja, zh-hans, zh-hant, th");
                return;
            }
            None => {
                let _ = load_default_parsers();
                eprintln!("Missing --lang value.");
                return;
            }
        }
    };

    let chunks = parser.parse(&input);
    println!("{}", chunks.join(&separator));
}

fn read_stdin() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input.trim_end().to_string())
}
