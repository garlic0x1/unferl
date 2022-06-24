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
    /// only print unique results
    #[clap(short, long)]
    unique: bool,
    /// print errors to stderr
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// print query keys{n}
    Keys,
    /// print query values{n}
    Values,
    /// print keypairs ("key=value"){n}
    Keypairs,
    /// print domains{n}
    Domains,
    /// print paths{n}
    Paths,
    /// printf style formatting (`unferl help format`){n}
    Format {
        /// printf style formatting{n}
        /// %% -> literal "%"{n}
        /// %s -> scheme{n}
        /// %d -> domain{n}
        /// %p -> path{n}
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
                }
                'p' => {
                    if fmt {
                        result.push_str(&self.path);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                }
                's' => {
                    if fmt {
                        result.push_str(&self.scheme);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                }
                'd' => {
                    if fmt {
                        result.push_str(&self.domain);
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                }
                _ => {
                    if fmt {
                        fmt = false;
                    } else {
                        result.push(c);
                    }
                }
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
                    if !filter.contains(key) || !args.unique {
                        println!("{}", key);
                        filter.insert(key.to_string());
                    }
                }
            }
            Commands::Values => {
                for value in parsed.values.iter() {
                    if !filter.contains(value) || !args.unique {
                        println!("{}", value);
                        filter.insert(value.to_string());
                    }
                }
            }
            Commands::Keypairs => {
                for (i, value) in parsed.values.iter().enumerate() {
                    if let Some(key) = parsed.keys.get(i) {
                        let pair = format!("{}={}", key, value);
                        if !filter.contains(&pair) || !args.unique {
                            println!("{}", pair);
                            filter.insert(pair);
                        }
                    }
                }
            }
            Commands::Domains => {
                if !filter.contains(&parsed.domain) || !args.unique {
                    println!("{}", parsed.domain);
                    filter.insert(parsed.domain);
                }
            }
            Commands::Paths => {
                if !filter.contains(&parsed.path) || !args.unique {
                    println!("{}", parsed.path);
                    filter.insert(parsed.path);
                }
            }
            Commands::Format { value } => {
                if let Some(string) = value {
                    let formatted = parsed.format(&string);
                    if !filter.contains(&parsed.path) || !args.unique {
                        println!("{}", formatted);
                        filter.insert(formatted);
                    }
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
                    if let Err(e) = tx.send(p) {
                        if args.verbose {
                            eprintln!("Error parsing URL: {}", e);
                        }
                    }
                };
            }
        }
    });

    writer(rx, &args);
}
