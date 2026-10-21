use super::clamp_origin;

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
