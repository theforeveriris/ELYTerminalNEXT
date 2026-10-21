//! SSH hosts from the user's config, and the values ssh itself resolves.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use gpui::Global;

/// One host the user's `~/.ssh/config` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Host {
    /// The name `ssh` takes, the Host line's first word.
    pub name: String,
    /// The address, the name when the block names none.
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity: Vec<String>,
    pub proxy_jump: Option<String>,
    /// The file whose block named it.
    pub source: PathBuf,
}

impl Host {
    /// How a session row describes the host: `user@host:port`, with its jump when it has one.
    pub fn summary(&self) -> String {
        let user = self.user.as_deref().unwrap_or("");
        let port = self.port.map_or(String::new(), |port| format!(":{port}"));
        let jump = self
            .proxy_jump
            .as_ref()
            .map_or(String::new(), |jump| format!(" via {jump}"));
        if user.is_empty() {
            format!("{}{port}{jump}", self.hostname)
        } else {
            format!("{user}@{}{port}{jump}", self.hostname)
        }
    }
}

/// The hosts from the user's config, and the files it read.
#[derive(Clone, Debug, Default)]
pub struct Hosts {
    pub hosts: Vec<Host>,
    pub files: Vec<PathBuf>,
}

impl Global for Hosts {}

impl Hosts {
    /// The hosts whose name, address or user holds `query`, the name's match first.
    pub fn search(&self, query: &str) -> Vec<&Host> {
        let query = query.to_lowercase();
        let mut found: Vec<&Host> = self
            .hosts
            .iter()
            .filter(|host| {
                host.name.to_lowercase().contains(&query)
                    || host.hostname.to_lowercase().contains(&query)
                    || host
                        .user
                        .as_ref()
                        .is_some_and(|user| user.to_lowercase().contains(&query))
            })
            .collect();
        found.sort_by_key(|host| {
            (
                !host.name.to_lowercase().starts_with(&query),
                host.name.clone(),
            )
        });
        found
    }
}

/// Where the user's config lives.
pub fn default_config() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ssh").join("config"))
}

/// The hosts `path` and every file it includes name; a missing file names none.
pub fn read(path: &Path) -> Hosts {
    let mut hosts = Hosts::default();
    let mut seen = HashSet::new();
    load_into(path, &mut hosts, &mut seen, 0);
    hosts
        .hosts
        .sort_by(|left, right| left.name.cmp(&right.name));
    hosts.hosts.dedup_by(|left, right| left.name == right.name);
    hosts
}

fn load_into(path: &Path, hosts: &mut Hosts, seen: &mut HashSet<PathBuf>, depth: usize) {
    if depth > 8 {
        log::warn!("ssh: {} is included too deeply", path.display());
        return;
    }
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if !seen.insert(path.clone()) {
        log::warn!("ssh: {} includes itself", path.display());
        return;
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            log::debug!("ssh: no config at {}", path.display());
            return;
        }
        Err(error) => {
            log::warn!("ssh: cannot read {}: {error}", path.display());
            return;
        }
    };
    hosts.files.push(path.clone());
    let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
    for pattern in includes(&text) {
        for file in expand(&pattern, &base) {
            load_into(&file, hosts, seen, depth + 1);
        }
    }
    for host in parse(&text, &path) {
        match hosts.hosts.iter_mut().find(|other| other.name == host.name) {
            Some(other) => other.fill_from(host),
            None => hosts.hosts.push(host),
        }
    }
}

impl Host {
    /// Fills what it leaves unset from a later block, which ssh reads next.
    fn fill_from(&mut self, later: Host) {
        if self.hostname == self.name {
            self.hostname = later.hostname;
        }
        if self.user.is_none() {
            self.user = later.user;
        }
        if self.port.is_none() {
            self.port = later.port;
        }
        if self.identity.is_empty() {
            self.identity = later.identity;
        }
        if self.proxy_jump.is_none() {
            self.proxy_jump = later.proxy_jump;
        }
    }
}

/// Every `Include` in `text`, each pattern as written.
pub fn includes(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = split(line) else {
            continue;
        };
        if key.eq_ignore_ascii_case("include") {
            found.extend(value.split_whitespace().map(str::to_string));
        }
    }
    found
}

/// The hosts `text` names, one for each concrete pattern in each Host block.
pub fn parse(text: &str, source: &Path) -> Vec<Host> {
    let mut hosts = Vec::new();
    let mut block: Option<Vec<Host>> = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        let Some((key, value)) = split(line) else {
            continue;
        };
        if key.eq_ignore_ascii_case("host") {
            hosts.extend(block.take().unwrap_or_default());
            let names: Vec<String> = value
                .split_whitespace()
                .filter(|name| !name.contains(['*', '?', '!']))
                .map(str::to_string)
                .collect();
            block = Some(
                names
                    .into_iter()
                    .map(|name| Host {
                        name: name.clone(),
                        hostname: name,
                        user: None,
                        port: None,
                        identity: Vec::new(),
                        proxy_jump: None,
                        source: source.to_path_buf(),
                    })
                    .collect(),
            );
        } else if key.eq_ignore_ascii_case("match") {
            hosts.extend(block.take().unwrap_or_default());
        } else if let Some(block) = block.as_mut() {
            for host in block {
                host.set(key, value);
            }
        }
    }
    hosts.extend(block.unwrap_or_default());
    hosts
}

impl Host {
    fn set(&mut self, key: &str, value: &str) {
        if key.eq_ignore_ascii_case("hostname") && self.hostname == self.name {
            self.hostname = value.to_string();
        } else if key.eq_ignore_ascii_case("user") && self.user.is_none() {
            self.user = Some(value.to_string());
        } else if key.eq_ignore_ascii_case("port") && self.port.is_none() {
            match value.parse() {
                Ok(port) => self.port = Some(port),
                Err(_) => log::warn!("ssh: {} names a port that is no number: {value}", self.name),
            }
        } else if key.eq_ignore_ascii_case("identityfile") {
            self.identity.push(value.to_string());
        } else if key.eq_ignore_ascii_case("proxyjump") && self.proxy_jump.is_none() {
            self.proxy_jump = Some(value.to_string());
        }
    }
}

/// `line`'s first word and the rest, trimmed.
fn split(line: &str) -> Option<(&str, &str)> {
    let at = line.find(char::is_whitespace)?;
    let (key, value) = line.split_at(at);
    let value = value.trim();
    (!key.is_empty() && !value.is_empty()).then_some((key, value))
}

/// The files `pattern` names under `base`: `~` is the home folder, and `*` and `?` match in each name.
pub fn expand(pattern: &str, base: &Path) -> Vec<PathBuf> {
    let path = if let Some(rest) = pattern.strip_prefix("~/") {
        match dirs::home_dir() {
            Some(home) => home.join(rest),
            None => return Vec::new(),
        }
    } else {
        let path = Path::new(pattern);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        }
    };
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    if !name.contains(['*', '?']) {
        return path.exists().then_some(path).into_iter().collect();
    }
    let folder = path.parent().unwrap_or(base);
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|file| {
            file.file_name()
                .and_then(|other| other.to_str())
                .is_some_and(|other| matches(name, other))
        })
        .collect();
    found.sort();
    found
}

/// Whether `text` matches `pattern`, where `*` takes any run and `?` one character.
pub fn matches(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let (mut at, mut in_text) = (0, 0);
    let (mut star, mut retry) = (None, 0);
    while in_text < text.len() {
        if at < pattern.len() && (pattern[at] == '?' || pattern[at] == text[in_text]) {
            at += 1;
            in_text += 1;
        } else if at < pattern.len() && pattern[at] == '*' {
            star = Some(at);
            retry = in_text;
            at += 1;
        } else if let Some(found) = star {
            at = found + 1;
            retry += 1;
            in_text = retry;
        } else {
            return false;
        }
    }
    while at < pattern.len() && pattern[at] == '*' {
        at += 1;
    }
    at == pattern.len()
}

/// What `ssh -G` resolves a host to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effective {
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity: Vec<String>,
    pub proxy_jump: Option<String>,
    pub local_forward: Vec<String>,
    pub remote_forward: Vec<String>,
}

impl Effective {
    /// `user@host:port`, as a status line shows it.
    pub fn summary(&self) -> String {
        let user = self.user.as_deref().unwrap_or("");
        let port = self.port.map_or(String::new(), |port| format!(":{port}"));
        if user.is_empty() {
            format!("{}{port}", self.hostname)
        } else {
            format!("{user}@{}{port}", self.hostname)
        }
    }
}

/// The effective settings in an `ssh -G` answer.
pub fn effective(output: &str) -> Effective {
    let mut out = Effective::default();
    for line in output.lines() {
        let Some((key, value)) = split(line) else {
            continue;
        };
        match key.to_ascii_lowercase().as_str() {
            "hostname" => out.hostname = value.to_string(),
            "user" => out.user = Some(value.to_string()),
            "port" => out.port = value.parse().ok(),
            "identityfile" => out.identity.push(value.to_string()),
            "proxyjump" => out.proxy_jump = (value != "none").then(|| value.to_string()),
            "localforward" => out.local_forward.push(value.to_string()),
            "remoteforward" => out.remote_forward.push(value.to_string()),
            _ => {}
        }
    }
    out
}

/// Resolves hosts through `ssh -G`, keeping answers until the config changes.
pub struct Resolver {
    config: PathBuf,
    stamp: Option<SystemTime>,
    resolved: HashMap<String, Effective>,
}

impl Resolver {
    pub fn new(config: PathBuf) -> Self {
        Self {
            config,
            stamp: None,
            resolved: HashMap::new(),
        }
    }

    /// What ssh resolves `host` to, from the cache while the config is unchanged.
    pub fn get(&mut self, host: &str) -> Option<&Effective> {
        let stamp = fs::metadata(&self.config)
            .and_then(|meta| meta.modified())
            .ok();
        if stamp != self.stamp {
            self.stamp = stamp;
            self.resolved.clear();
        }
        if !self.resolved.contains_key(host) {
            let answer = Command::new("ssh").args(["-G", host]).output().ok()?;
            if !answer.status.success() {
                log::debug!(
                    "ssh: -G {host} failed: {}",
                    String::from_utf8_lossy(&answer.stderr)
                );
                return None;
            }
            let text = String::from_utf8_lossy(&answer.stdout);
            self.resolved.insert(host.to_string(), effective(&text));
        }
        self.resolved.get(host)
    }
}

#[cfg(test)]
mod tests;
