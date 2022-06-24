use url::Url;

pub struct ParsedUrl {
    pub parsed: Url,
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub keys: Vec<String>,
    pub values: Vec<String>,
    pub query: String,
    pub fragment: String,
    pub port: Option<u16>,
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
                Ok(Self {
                    scheme: u.scheme().to_string(),
                    domain: u.host().unwrap().to_string(),
                    path: u.path().to_string(),
                    port: u.port(),
                    fragment: u.fragment().unwrap_or_default().to_string(),
                    keys,
                    values,
                    query: u.query().unwrap_or_default().to_string(),
                    parsed: u,
                })
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
                            if u.len() > 0 {
                                result.push_str(u);
                                result.push('@');
                            }
                            result.push_str(&self.domain);
                            if let Some(p) = &self.port {
                                result.push(':');
                                result.push_str(p.to_string().as_str());
                            }
                        }
                    }
                    '@' => {
                        if let Some(s) = &self.user_info() {
                            if s.len() > 0 {
                                result.push('@');
                            }
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
