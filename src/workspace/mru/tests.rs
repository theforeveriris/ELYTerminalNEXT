use super::*;

#[test]
fn a_shown_session_moves_to_the_front() {
    let mut mru = Mru::default();
    for id in [1, 2, 3] {
        mru.touch(id);
    }
    assert_eq!(mru.next(3, true), Some(2));
    mru.touch(1);
    assert_eq!(mru.next(1, true), Some(3), "the newest other one");
    assert_eq!(mru.next(1, false), Some(2), "the oldest other one");
}

#[test]
fn a_closed_session_is_forgotten() {
    let mut mru = Mru::default();
    mru.touch(1);
    mru.touch(2);
    mru.forget(2);
    assert_eq!(mru.next(1, true), None, "no other session");
}

#[test]
fn one_session_alone_has_nowhere_to_go() {
    let mut mru = Mru::default();
    assert_eq!(mru.next(1, true), None);
    mru.touch(1);
    assert_eq!(mru.next(1, true), None);
    assert_eq!(mru.next(1, false), None);
}
