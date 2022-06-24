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

struct ParsedUrl {
    parsed: Url,
    scheme: String,
    domain: String,
    path: String,
    keys: Vec<String>,
    values: Vec<String>,
    query: String,
    fragment: String,
    port: Option<u16>,
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
                    port: u.port(),
                    fragment: u.fragment().unwrap_or_default().to_string(),
                    keys,
                    values,
                    query: u.query().unwrap_or_default().to_string(),
                    parsed: u,
                };

                Ok(parsed)
            }
            Err(_e) => Err(String::from("failed to parse URL")),
        }
    }

    pub fn user_info(&self) -> Option<String> {
        if self.parsed.has_authority() {
            let mut result = String::new();
            result.push_str(self.parsed.username());
            if let Some(p) = self.parsed.password() {
                result.push(':');
                result.push_str(p);
            }
            Some(result)
        } else {
            None
        }
    }

    pub fn file_extension(&self) -> Option<String> {
        let v: Vec<&str> = self.path.as_str().splitn(2, '/').collect();
        if let Some(f) = v.get(v.len() - 1) {
            let v2: Vec<&str> = f.splitn(2, '.').collect();
            if let Some(ext) = v2.get(1) {
                return Some(ext.to_string());
            }
        }
        None
    }

    pub fn subdomain(&self) -> String {
        let v: Vec<&str> = self.domain.as_str().split('.').collect();
        v[0..(v.len() - 2)].join(".").to_string()
    }

    pub fn root_domain(&self) -> String {
        let v: Vec<&str> = self.domain.as_str().split('.').collect();
        v[(v.len() - 2)..v.len()].join(".").to_string()
    }

    pub fn tld(&self) -> String {
        let v: Vec<&str> = self.domain.as_str().split('.').collect();
        v[v.len() - 1].to_string()
    }

    pub fn format(&self, format_string: &str) -> String {
        let mut result = String::new();
        let mut fmt = false;
        for c in format_string.chars() {
            if c == '%' {
                if fmt {
                    result.push(c);
                }
                fmt = !fmt;
                continue;
            }
            if fmt {
                match c {
                    's' => {
                        result.push_str(&self.scheme);
                    }
                    'd' => {
                        result.push_str(&self.domain);
                    }
                    'S' => {
                        result.push_str(&self.subdomain());
                    }
                    'r' => {
                        result.push_str(&self.root_domain());
                    }
                    't' => {
                        result.push_str(&self.tld());
                    }
                    'p' => {
                        result.push_str(&self.path);
                    }
                    'e' => {
                        if let Some(ext) = &self.file_extension() {
                            result.push_str(ext);
                        }
                    }
                    'q' => {
                        result.push_str(&self.query);
                    }
                    '?' => {
                        if self.query.len() > 0 {
                            result.push('?');
                        }
                    }
                    'f' => {
                        result.push_str(&self.fragment);
                    }
                    '#' => {
                        if self.fragment.len() > 0 {
                            result.push('#');
                        }
                    }
                    'P' => {
                        if let Some(p) = &self.port {
                            result.push_str(p.to_string().as_str());
                        }
                    }
                    ':' => {
                        if let Some(_) = &self.port {
                            result.push(':');
                        }
                    }
                    'u' => {
                        if let Some(u) = &self.user_info() {
                            result.push_str(u);
                        }
                    }
                    'a' => {
                        if let Some(u) = &self.user_info() {
                            result.push_str(u);
                            result.push('@');
                            result.push_str(&self.domain);
                            if let Some(p) = &self.port {
                                result.push(':');
                                result.push_str(p.to_string().as_str());
                            }
                        }
                    }
                    '@' => {
                        if let Some(_) = &self.user_info() {
                            result.push('@');
                        }
                    }
                    _ => (),
                }
                fmt = false;
            } else {
                result.push(c)
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
