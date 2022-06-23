use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;
use std::{io, io::prelude::*};
use url::Url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Arguments {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long)]
    unique: bool,
}

#[derive(Subcommand)]
enum Commands {
    Keys,
    Values,
    Domains,
    Paths,
    Format {
        #[clap(value_parser)]
        value: Option<String>,
    },
}

struct ParsedUrl {
    scheme: String,
    domain: String,
    path: String,
    keys: Vec<String>,
    values: Vec<String>,
}

impl ParsedUrl {
    pub fn new(line: String) -> Result<Self, String> {
        match Url::parse(line.as_str()) {
            Ok(u) => {
                let mut keys = Vec::new();
                let mut values = Vec::new();
                for pair in u.query_pairs() {
                    keys.push(pair.0.to_string());
                    values.push(pair.1.to_string());
                }
                let parsed = Self {
                    scheme: u.scheme().to_string(),
                    domain: u.host().unwrap().to_string(),
                    path: u.path().to_string(),
                    keys,
                    values,
                };

                Ok(parsed)
            }
            Err(_e) => Err(String::from("failed to parse URL")),
        }
    }

    pub fn format(&self, format_string: &str) -> String {
        let mut result = String::new();
        let mut fmt = false;
        for c in format_string.chars() {
            match c {
                '%' => {
                    if fmt {
                        result.push(c);
                    }
                    fmt = !fmt;
                },
                'p' => {
                    if fmt {
                        result.push_str(&self.path);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                },
                's' => {
                    if fmt {
                        result.push_str(&self.scheme);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                },
                'd' => {
                    if fmt {
                        result.push_str(&self.domain);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                },
                _ => {
                    if fmt {
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                },
            }
        }
        result
    }
}

fn writer(rx: mpsc::Receiver<ParsedUrl>, args: &Arguments) {
    let mut filter: HashSet<String> = HashSet::new();
    for parsed in rx {
        match &args.command {
            Commands::Keys => {
                for key in parsed.keys.iter() {
                    if !filter.contains(key) {
                        println!("{}", key);
                        filter.insert(key.to_string());
                    }
                }
            }
            Commands::Values => {
                for value in parsed.values.iter() {
                    if !filter.contains(value) {
                        println!("{}", value);
                        filter.insert(value.to_string());
                    }
                }
            }
            Commands::Domains => {
                if !filter.contains(&parsed.domain) {
                    println!("{}", parsed.domain);
                    filter.insert(parsed.domain);
                }
            }
            Commands::Paths => {
                if !filter.contains(&parsed.path) {
                    println!("{}", parsed.path);
                    filter.insert(parsed.path);
                }
            }
            Commands::Format { value } => {
                if let Some(string) = value {
                    println!("{}", parsed.format(&string));
                }
            }
        }
    }
}

fn main() {
    let args = Arguments::parse();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            if let Ok(l) = line {
                let parsed = ParsedUrl::new(l);
                if let Ok(p) = parsed {
                    match tx.send(p) {
                        Ok(..) => (),
                        Err(..) => (),
                    }
                };
            }
        }
    });

    writer(rx, &args);
}
