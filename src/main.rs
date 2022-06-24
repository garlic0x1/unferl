use unferl::ParsedUrl;
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;
use std::{io, io::prelude::*};


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
    /// print fragments{n}
    Fragments,
    /// printf style formatting (`unferl help format`){n}
    Format {
        /// printf style formatting{n}
        /// %% -> literal "%"{n}
        /// %s -> scheme{n}
        /// %u -> user info (user:pass){n}
        /// %a -> authority (alias for %u%@%d%:%P){n}
        /// %d -> domain{n}
        /// %S -> subdomain{n}
        /// %r -> root domain{n}
        /// %t -> TLD (".com", ".org", etc){n}
        /// %P -> port{n}
        /// %p -> path{n}
        /// %q -> query string{n}
        /// %f -> fragment{n}
        /// %@ -> insert @ if user info specified{n}
        /// %: -> insert : if port specified{n}
        /// %? -> insert ? if query exists{n}
        /// %# -> insert # if fragment exists{n}
        #[clap(value_parser)]
        value: Option<String>,
    },
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
            Commands::Fragments => {
                if !filter.contains(&parsed.fragment) || !args.unique {
                    println!("{}", parsed.fragment);
                    filter.insert(parsed.fragment);
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
                } else {
                    eprintln!("provide a format string, Example: %s://%d%p");
                    return;
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
                match ParsedUrl::new(l) {
                    Ok(p) => {
                        if let Err(e) = tx.send(p) {
                            if args.verbose {
                                eprintln!("Error sending on chan: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        if args.verbose {
                            eprintln!("Error parsing URL: {}", e);
                        }
                    }
                }
            }
        }
    });

    writer(rx, &args);
}
