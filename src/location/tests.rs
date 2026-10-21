use super::*;

#[test]
fn a_percent_escape_decodes_or_gives_none() {
    assert_eq!(
        percent_decode(b"/tmp/ely%20dir").as_deref(),
        Some("/tmp/ely dir")
    );
    assert_eq!(
        percent_decode("/%E4%B8%AD".as_bytes()).as_deref(),
        Some("/中")
    );
    assert_eq!(percent_decode(b"/%FF"), None);
    assert_eq!(percent_decode(b"/%zz"), Some("/%zz".into()));
    assert_eq!(percent_decode(b"/100%"), Some("/100%".into()));
}

#[test]
fn a_file_uri_from_this_machine_is_a_local_folder() {
    let local = "workstation";
    assert_eq!(
        Location::from_file_uri("file:///tmp/a%20b", local),
        Some(Location::Local(PathBuf::from("/tmp/a b")))
    );
    assert_eq!(
        Location::from_file_uri("file://localhost/x", local),
        Some(Location::Local(PathBuf::from("/x")))
    );
    assert_eq!(
        Location::from_file_uri("file://WORKSTATION/y", local),
        Some(Location::Local(PathBuf::from("/y")))
    );
    assert_eq!(Location::from_file_uri("file://elsewhere/x", local), None);
    assert_eq!(Location::from_file_uri("https://x/y", local), None);
    assert_eq!(Location::from_file_uri("file:///%FF", local), None);
}

#[cfg(windows)]
#[test]
fn a_windows_drive_uri_loses_its_leading_slash() {
    assert_eq!(
        Location::from_file_uri("file:///C:/Users/me", "me"),
        Some(Location::Local(PathBuf::from(r"C:\Users\me")))
    );
}

#[test]
fn a_windows_path_reads_as_wsl_does() {
    assert_eq!(
        windows_to_wsl(r"C:\Users\me\src").as_deref(),
        Some("/mnt/c/Users/me/src")
    );
    assert_eq!(windows_to_wsl("D:/work").as_deref(), Some("/mnt/d/work"));
    assert_eq!(windows_to_wsl(r"C:\").as_deref(), Some("/mnt/c"));
    assert_eq!(
        windows_to_wsl(r"\\wsl.localhost\Ubuntu-22.04\home\me").as_deref(),
        Some("/home/me")
    );
    assert_eq!(
        windows_to_wsl(r"\\wsl$\archlinux\srv").as_deref(),
        Some("/srv")
    );
    assert_eq!(windows_to_wsl(r"\\server\share"), None);
    assert_eq!(windows_to_wsl("relative"), None);
    assert_eq!(windows_to_wsl(r"C:relative"), None);
}

#[test]
fn a_wsl_folder_reads_as_windows_does() {
    assert_eq!(
        wsl_to_windows("/mnt/c/Users/me", "Ubuntu-22.04").as_deref(),
        Some(r"C:\Users\me")
    );
    assert_eq!(
        wsl_to_windows("/mnt/d", "Ubuntu-22.04").as_deref(),
        Some(r"D:\")
    );
    assert_eq!(
        wsl_to_windows("/home/me", "archlinux").as_deref(),
        Some(r"\\wsl.localhost\archlinux\home\me")
    );
    assert_eq!(wsl_to_windows("relative", "archlinux"), None);
}

#[test]
fn a_git_bash_path_and_a_windows_path_round_trip() {
    assert_eq!(
        windows_to_posix(r"C:\Program Files\Git").as_deref(),
        Some("/c/Program Files/Git")
    );
    assert_eq!(windows_to_posix(r"C:\").as_deref(), Some("/c"));
    assert_eq!(windows_to_posix(r"\\server\share"), None);
    assert_eq!(
        posix_to_windows("/c/Program Files/Git").as_deref(),
        Some(r"C:\Program Files\Git")
    );
    assert_eq!(posix_to_windows("/c").as_deref(), Some(r"C:\"));
    assert_eq!(
        posix_to_windows("/usr/bin"),
        None,
        "the installation root is not a drive"
    );
    assert_eq!(posix_to_windows("relative"), None);
}

#[test]
fn a_location_shows_where_it_is() {
    assert_eq!(Location::local(r"C:\src").display(), r"C:\src");
    let wsl = Location::Wsl {
        distro: "archlinux".into(),
        path: "/home/u".into(),
    };
    assert_eq!(wsl.display(), "WSL: archlinux:/home/u");
    assert_eq!(
        wsl.windows_path(),
        Some(PathBuf::from(r"\\wsl.localhost\archlinux\home\u"))
    );
    let ssh = Location::Ssh {
        host: "example.com".into(),
        user: Some("me".into()),
        path: "/srv".into(),
    };
    assert_eq!(ssh.display(), "me@example.com:/srv");
    assert_eq!(ssh.on_disk(), None);
    assert_eq!(ssh.windows_path(), None);
    assert_eq!(
        Location::Container {
            name: "api".into(),
            path: "/app".into()
        }
        .display(),
        "api:/app"
    );
}
