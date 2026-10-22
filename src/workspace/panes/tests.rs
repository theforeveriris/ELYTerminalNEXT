use crate::workspace::layout::Tab;

#[test]
fn a_share_is_eight_pixels_to_a_percent() {
    let share = |pixels: f32| (pixels / 8.0).round() as i16;
    assert_eq!(share(0.0), 0);
    assert_eq!(share(8.0), 1);
    assert_eq!(share(-16.0), -2);
    assert_eq!(share(79.0), 10, "the nearest percent");
}

#[test]
fn a_dragged_row_and_a_double_press_leave_the_panes_even() {
    let mut tab = Tab::new(0);
    tab.split(true, 1);
    tab.widen(1, 20);
    assert_eq!((tab.panes[0].width, tab.panes[1].width), (30, 70));
    assert!(tab.even(1));
    assert_eq!((tab.panes[0].width, tab.panes[1].width), (50, 50));
}
