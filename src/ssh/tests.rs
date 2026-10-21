use std::{env, fs, path::PathBuf};

use super::*;

fn folder(name: &str) -> PathBuf {
    let folder = env::temp_dir().join(format!("elyterminal-ssh-{name}-{}", std::process::id()));
    if folder.exists() {
        fs::remove_dir_all(&folder).expect("a stale folder goes");
    }
    fs::create_dir_all(&folder).expect("a test folder");
    folder
}

#[test]
fn a_config_names_its_hosts_with_their_values() {
    let text = "\
# a comment
Host work
  HostName work.example.com
  User me
  Port 2222
  IdentityFile ~/.ssh/work
  ProxyJump bastion

Host * 
  User nobody
";
    let hosts = parse(text, Path::new("/home/me/.ssh/config"));
    assert_eq!(hosts.len(), 1, "the wildcard block names no host");
    let host = &hosts[0];
    assert_eq!(host.name, "work");
    assert_eq!(host.hostname, "work.example.com");
    assert_eq!(host.user.as_deref(), Some("me"));
    assert_eq!(host.port, Some(2222));
    assert_eq!(host.identity, ["~/.ssh/work"]);
    assert_eq!(host.proxy_jump.as_deref(), Some("bastion"));
    assert_eq!(host.summary(), "me@work.example.com:2222 via bastion");
}

#[test]
fn the_first_value_wins_and_a_later_block_fills_what_is_missing() {
    let text = "\
Host work
  HostName first.example.com
Host work
  HostName second.example.com
  User me
";
    let hosts = parse(text, Path::new("config"));
    assert_eq!(hosts.len(), 2, "each block reads on its own");
    assert_eq!(hosts[0].hostname, "first.example.com");
    assert_eq!(hosts[1].hostname, "second.example.com");
    assert_eq!(hosts[1].user.as_deref(), Some("me"));
}

#[test]
fn one_block_names_every_concrete_pattern() {
    let text = "Host one two *.internal\n  User me\n";
    let hosts = parse(text, Path::new("config"));
    assert_eq!(hosts.len(), 2);
    assert_eq!(hosts[0].name, "one");
    assert_eq!(hosts[1].name, "two");
    assert_eq!(hosts[0].hostname, "one", "no HostName means the name");
    assert_eq!(hosts[1].user.as_deref(), Some("me"));
}

#[test]
fn a_match_ends_the_block() {
    let text = "Host work\n  User me\nMatch host other\n  User you\n";
    let hosts = parse(text, Path::new("config"));
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].user.as_deref(), Some("me"));
}

#[test]
fn a_config_includes_the_files_its_pattern_names() {
    let root = folder("include");
    let config = root.join("config");
    let conf = root.join("conf.d");
    fs::create_dir_all(&conf).expect("conf.d");
    fs::write(
        &config,
        "Include conf.d/*.conf\nHost main\n  HostName main.example.com\n",
    )
    .expect("config");
    fs::write(conf.join("a.conf"), "Host alpha\n  User a\n").expect("a");
    fs::write(conf.join("b.conf"), "Host beta\n  User b\n").expect("b");
    fs::write(conf.join("notes.txt"), "Host ignored\n").expect("notes");
    let hosts = read(&config);
    let names: Vec<&str> = hosts.hosts.iter().map(|host| host.name.as_str()).collect();
    assert_eq!(names, ["alpha", "beta", "main"]);
    assert_eq!(hosts.files.len(), 3, "the config and both includes");
    assert_eq!(hosts.search("alp")[0].name, "alpha");
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn an_include_cycle_stops() {
    let root = folder("cycle");
    let a = root.join("a");
    let b = root.join("b");
    fs::write(&a, "Include b\nHost one\n").expect("a");
    fs::write(&b, "Include a\nHost two\n").expect("b");
    let hosts = read(&a);
    assert_eq!(hosts.hosts.len(), 2);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_wildcard_takes_a_run_and_a_question_one_character() {
    assert!(matches("*.conf", "a.conf"));
    assert!(matches("conf.d/*", "conf.d/work"));
    assert!(!matches("*.conf", "a.txt"));
    assert!(matches("a?c", "abc"));
    assert!(!matches("a?c", "ac"));
    assert!(matches("*", ""));
    assert!(matches("a*b*c", "axxbyyc"));
    assert!(!matches("a*b*c", "axxbyy"));
}

#[test]
fn ssh_g_s_answer_reads_to_its_values() {
    let output = "\
host work
hostname work.example.com
user me
port 2222
identityfile ~/.ssh/id_ed25519
proxyjump bastion
localforward 8080 localhost:80
remoteforward 9090 localhost:90
";
    let effective = effective(output);
    assert_eq!(effective.summary(), "me@work.example.com:2222");
    assert_eq!(effective.identity, ["~/.ssh/id_ed25519"]);
    assert_eq!(effective.proxy_jump.as_deref(), Some("bastion"));
    assert_eq!(effective.local_forward, ["8080 localhost:80"]);
    assert_eq!(effective.remote_forward, ["9090 localhost:90"]);
}

#[test]
fn search_names_the_host_that_starts_with_the_query_first() {
    let hosts = Hosts {
        hosts: vec![
            Host {
                name: "work".into(),
                hostname: "work.example.com".into(),
                user: Some("me".into()),
                port: None,
                identity: Vec::new(),
                proxy_jump: None,
                source: PathBuf::new(),
            },
            Host {
                name: "office".into(),
                hostname: "work2.example.com".into(),
                user: None,
                port: None,
                identity: Vec::new(),
                proxy_jump: None,
                source: PathBuf::new(),
            },
        ],
        files: Vec::new(),
    };
    let found = hosts.search("work");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].name, "work");
    assert_eq!(found[1].name, "office");
}

#[test]
fn a_relative_include_stays_under_the_config_s_folder() {
    let root = folder("relative");
    let config = root.join("config");
    fs::create_dir_all(root.join("parts")).expect("parts");
    fs::write(root.join("parts").join("one"), "Host one\n").expect("one");
    fs::write(&config, "Include parts/one\n").expect("config");
    let hosts = read(&config);
    assert_eq!(hosts.hosts.len(), 1);
    assert_eq!(hosts.hosts[0].name, "one");
    fs::remove_dir_all(&root).expect("cleanup");
}
