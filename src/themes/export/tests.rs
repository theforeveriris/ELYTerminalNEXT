use super::*;
use crate::themes::import;

#[test]
fn an_exported_theme_reads_back_as_it_was() {
    let colors = Colors::default();
    let text = ghostty(&colors);
    assert!(text.starts_with("palette = 0=#45475a\n"));
    assert!(text.contains("selection-foreground = #cdd6f4\n"));
    assert_eq!(import::ghostty(&text).expect("reads"), colors);
}

#[test]
fn a_theme_file_names_all_sixteen_palette_colors() {
    let text = ghostty(&Colors::default());
    for index in 0..16 {
        assert!(
            text.contains(&format!("palette = {index}=")),
            "palette {index}"
        );
    }
}
