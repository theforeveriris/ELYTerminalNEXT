use super::*;

fn joined(template: &str, path: &str) -> Option<(String, Vec<String>)> {
    command(template, Path::new(path)).map(|(program, args)| {
        (
            program,
            args.into_iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        )
    })
}

#[test]
fn a_command_line_splits_on_spaces_unless_quoted() {
    assert_eq!(words("code -g"), ["code", "-g"]);
    assert_eq!(
        words(r#""C:\Program Files\Code\Code.exe" -g"#),
        [r"C:\Program Files\Code\Code.exe".to_string(), "-g".into()]
    );
    assert_eq!(words("'a b' c"), ["a b", "c"]);
    assert_eq!(words("a\\ b c"), ["a b", "c"]);
    assert_eq!(words("   "), Vec::<String>::new());
}

#[test]
fn the_file_goes_where_file_is_or_after_the_arguments() {
    assert_eq!(
        joined("code -g {file}:12", "/a/b.txt"),
        Some(("code".into(), vec!["-g".into(), "/a/b.txt:12".into()]))
    );
    assert_eq!(
        joined("zed", "/a/b.txt"),
        Some(("zed".into(), vec!["/a/b.txt".into()]))
    );
    assert_eq!(joined("", "/a/b.txt"), None);
}
