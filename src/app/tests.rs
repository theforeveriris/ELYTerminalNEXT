use super::*;

#[test]
fn a_window_opens_maximized_when_it_settled_that_way() {
    let bounds = Bounds::new(point(px(10.0), px(20.0)), size(px(800.0), px(600.0)));
    assert!(matches!(
        opening_window_bounds(true, bounds),
        WindowBounds::Maximized(_)
    ));
    assert!(matches!(
        opening_window_bounds(false, bounds),
        WindowBounds::Windowed(_)
    ));
}

#[test]
fn a_window_kept_inside_the_screen_stays_where_it_was() {
    assert_eq!(
        clamp_origin((100.0, 50.0), (800.0, 600.0), (0.0, 0.0, 1920.0, 1080.0)),
        (100.0, 50.0)
    );
}

#[test]
fn a_window_kept_off_the_screen_comes_back_to_its_edge() {
    assert_eq!(
        clamp_origin((-40.0, -20.0), (800.0, 600.0), (0.0, 0.0, 1920.0, 1080.0)),
        (0.0, 0.0)
    );
    assert_eq!(
        clamp_origin((1500.0, 900.0), (800.0, 600.0), (0.0, 0.0, 1920.0, 1080.0)),
        (1120.0, 480.0)
    );
}

#[test]
fn a_window_larger_than_the_screen_starts_at_its_corner() {
    assert_eq!(
        clamp_origin((500.0, 300.0), (2000.0, 1200.0), (0.0, 0.0, 1920.0, 1080.0)),
        (0.0, 0.0)
    );
}

#[test]
fn a_screen_that_starts_below_the_origin_keeps_the_window_past_it() {
    assert_eq!(
        clamp_origin((-100.0, 0.0), (800.0, 600.0), (-100.0, 0.0, 1920.0, 1080.0)),
        (-100.0, 0.0)
    );
}
