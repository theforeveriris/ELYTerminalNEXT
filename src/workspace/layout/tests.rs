use super::*;

#[test]
fn a_tab_holds_the_sessions_of_its_panes() {
    let mut tab = Tab::new(3);
    assert!(tab.holds(3));
    assert!(!tab.holds(4));
    tab.split(true, 4);
    assert!(tab.holds(3) && tab.holds(4));
}

#[test]
fn a_new_tab_holds_one_pane_with_the_focus() {
    let tab = Tab::new(3);
    assert_eq!(tab.panes.len(), 1);
    assert_eq!(tab.focused, 0);
    assert_eq!(tab.focused_session(), Some(3));
    assert_eq!(tab.panes[0].session, 3);
    assert_eq!(
        tab.panes.get(1).map(|pane| pane.session),
        None,
        "no second pane yet"
    );
}

#[test]
fn a_tab_is_named_by_its_focused_session_unless_it_has_a_name() {
    let mut tab = Tab::new(0);
    assert_eq!(tab.named("shell", "ELYTerminal"), "shell");
    assert_eq!(tab.named("", "ELYTerminal"), "ELYTerminal");
    tab.name = "mine".into();
    assert_eq!(tab.named("shell", "ELYTerminal"), "mine");
}

#[test]
fn a_vertical_split_leaves_two_panes_side_by_side() {
    let mut tab = Tab::new(0);
    let new = tab.split(true, 0).expect("a pane");
    assert_eq!(new, 1);
    assert_eq!(tab.panes.len(), 2);
    assert_eq!(tab.focused, 1, "the new pane has the focus");
    assert_eq!(tab.rows(), vec![vec![0, 1]], "one row");
    assert_eq!(tab.panes[0].width, 50);
    assert_eq!(tab.panes[1].width, 50);
    assert_eq!(tab.panes[0].height, 100, "the height is untouched");
}

#[test]
fn a_horizontal_split_leaves_two_panes_above_one_another() {
    let mut tab = Tab::new(0);
    tab.split(false, 0);
    assert_eq!(tab.panes.len(), 2);
    assert_eq!(tab.rows(), vec![vec![0], vec![1]], "two rows");
    assert_eq!(tab.panes[0].height, 50);
    assert_eq!(tab.panes[1].height, 50);
}

#[test]
fn the_focus_walks_the_panes_in_their_order() {
    let mut tab = Tab::new(0);
    tab.split(true, 0);
    tab.focus(0);
    assert_eq!(tab.focus_neighbour(Direction::Right), Some(1));
    assert_eq!(
        tab.focus_neighbour(Direction::Left),
        None,
        "no pane to the left"
    );
    assert_eq!(tab.focus_neighbour(Direction::Up), None);
    assert_eq!(tab.focus_neighbour(Direction::Down), None, "one row only");
    tab.focus(1);
    assert_eq!(tab.focus_neighbour(Direction::Left), Some(0));
    assert_eq!(tab.focus_neighbour(Direction::Right), None);
}

#[test]
fn the_focus_walks_between_rows() {
    let mut tab = Tab::new(0);
    tab.split(false, 0);
    tab.focus(0);
    assert_eq!(tab.focus_neighbour(Direction::Down), Some(1));
    assert_eq!(tab.focus_neighbour(Direction::Up), None);
    tab.focus(1);
    assert_eq!(tab.focus_neighbour(Direction::Up), Some(0));
    assert_eq!(tab.focus_neighbour(Direction::Down), None);
}

#[test]
fn a_pane_that_goes_takes_the_focus_with_it() {
    let mut tab = Tab::new(0);
    tab.split(true, 0);
    tab.focus(1);
    assert!(!tab.remove(1), "a pane is left");
    assert_eq!(tab.focused, 0, "the focus fell back");
    assert!(tab.remove(0), "the last one went");
    assert!(tab.panes.is_empty());
}

#[test]
fn a_split_keeps_the_session_of_the_pane_it_came_from() {
    let mut tab = Tab::new(7);
    tab.split(true, 7);
    assert_eq!(tab.panes[0].session, 7);
    assert_eq!(tab.panes[1].session, 7, "until it gets one of its own");
}
#[test]
fn a_split_puts_its_own_session_in_the_new_pane() {
    let mut tab = Tab::new(7);
    tab.split(true, 9);
    assert_eq!(tab.panes[0].session, 7);
    assert_eq!(tab.panes[1].session, 9);
}

#[test]
fn a_pane_swaps_with_its_neighbour_and_the_focus_goes_with_it() {
    let mut tab = Tab::new(1);
    tab.split(true, 2);
    tab.focus(0);
    assert_eq!(tab.sessions(), vec![1, 2]);
    assert!(tab.swap(Direction::Right));
    assert_eq!(tab.sessions(), vec![2, 1]);
    assert_eq!(tab.focused_session(), Some(1));
    assert!(!tab.swap(Direction::Right), "no neighbour past the edge");
}

#[test]
fn a_tab_names_the_sessions_its_panes_hold() {
    let mut tab = Tab::new(1);
    assert_eq!(tab.sessions(), vec![1]);
    tab.split(true, 2);
    assert_eq!(tab.sessions(), vec![1, 2]);
    tab.split(false, 3);
    assert_eq!(tab.sessions(), vec![1, 2, 3]);
}

#[test]
fn a_drag_moves_a_share_between_neighbours_and_stops_at_the_least() {
    let mut tab = Tab::new(0);
    tab.split(true, 1);
    assert_eq!((tab.panes[0].width, tab.panes[1].width), (50, 50));
    assert!(tab.widen(1, 20), "the right pane grows");
    assert_eq!((tab.panes[0].width, tab.panes[1].width), (30, 70));
    assert!(tab.widen(1, -20), "and gives it back");
    assert_eq!((tab.panes[0].width, tab.panes[1].width), (50, 50));
    assert!(!tab.widen(0, 10), "the first pane has no left neighbour");
    assert!(tab.widen(1, 90));
    assert_eq!(tab.panes[0].width, LEAST, "the left pane keeps its least");
    assert_eq!(tab.panes[1].width, 100 - LEAST);
    assert!(!tab.widen(1, 90), "no room left to give");
}

#[test]
fn a_drag_between_rows_moves_their_heights_together() {
    let mut tab = Tab::new(0);
    tab.split(false, 1);
    assert_eq!((tab.panes[0].height, tab.panes[1].height), (50, 50));
    assert!(tab.heighten(1, 10));
    assert_eq!((tab.panes[0].height, tab.panes[1].height), (40, 60));
    assert!(!tab.heighten(0, 10), "the first row has none above it");
    tab.heighten(1, 100);
    assert_eq!(tab.panes[0].height, LEAST);
}

#[test]
fn a_tab_moves_among_its_neighbours_but_never_across_a_pinned_one() {
    let mut tabs: Vec<Tab> = (0..3u64).map(Tab::new).collect();
    assert!(Tab::move_by(&mut tabs, 1, -1), "the middle tab goes up");
    assert_eq!(tabs.len(), 3);
    assert!(!Tab::move_by(&mut tabs, 0, -1), "the first has nowhere up");
    assert!(!Tab::move_by(&mut tabs, 2, 1), "the last has nowhere down");
}

#[test]
fn a_pinned_tab_sits_before_the_rest_and_stays_put() {
    let mut tabs: Vec<Tab> = (0..3u64).map(Tab::new).collect();
    assert_eq!(Tab::pin(&mut tabs, 2, true), Some(0), "pinned, it leads");
    assert!(tabs[0].pinned);
    assert!(!Tab::move_by(&mut tabs, 0, 1), "a pinned tab does not move");
    assert!(!Tab::move_by(&mut tabs, 1, -1), "and none crosses it");
    assert!(Tab::move_by(&mut tabs, 2, -1), "the others still move");
    assert_eq!(
        Tab::pin(&mut tabs, 0, false),
        Some(0),
        "with no pinned tab left, it leads"
    );
    assert!(!tabs.iter().any(|tab| tab.pinned));
    assert_eq!(Tab::pin(&mut tabs, 2, true), Some(0));
    assert_eq!(
        Tab::pin(&mut tabs, 1, false),
        Some(1),
        "an unpinned tab goes after the pinned one"
    );
}

#[test]
fn a_row_and_the_rows_are_evened_again_at_once() {
    let mut tab = Tab::new(0);
    tab.split(true, 1);
    tab.split(false, 2);
    tab.widen(1, 20);
    assert!(tab.even(1));
    assert_eq!(tab.panes[0].width, 50);
    assert_eq!(tab.panes[1].width, 50);
    assert_eq!(tab.panes[0].height, 50);
    assert_eq!(tab.panes[2].height, 50);
    let mut alone = Tab::new(0);
    assert!(!alone.even(0), "a pane alone has nothing to even out");
}
