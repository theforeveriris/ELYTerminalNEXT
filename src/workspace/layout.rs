//! Tabs and panes: the workspace's tree of terminals.

/// One pane of a tab: a session, and how much of the tab it takes.
pub struct Pane {
    /// The session it shows, by its id, which never changes.
    pub session: SessionId,
    /// How much of the tab's width it takes, in tenths.
    pub width: u16,
    /// How much of the tab's height it takes, in tenths.
    pub height: u16,
}

/// A session's id, which stays the same as the session moves and closes.
pub type SessionId = u64;

impl Pane {
    /// A pane that takes the whole tab.
    pub fn whole(session: SessionId) -> Self {
        Self {
            session,
            width: 10,
            height: 10,
        }
    }
}

/// One tab of the window: a page of panes.
pub struct Tab {
    /// Its panes, in their order.
    pub panes: Vec<Pane>,
    /// The pane that has the focus, as an index into `panes`.
    pub focused: usize,
    /// The name the user gave it; empty leaves the title to the focused pane.
    pub name: String,
}

impl Tab {
    /// A tab with one pane, which has the focus.
    pub fn new(session: SessionId) -> Self {
        Self {
            panes: vec![Pane::whole(session)],
            focused: 0,
            name: String::new(),
        }
    }

    /// What it is called: the user's name, else the focused pane's session, else `fallback`.
    pub fn named<'a>(&'a self, session_name: &'a str, fallback: &'a str) -> &'a str {
        if !self.name.is_empty() {
            return &self.name;
        }
        if session_name.is_empty() {
            fallback
        } else {
            session_name
        }
    }

    /// The session in the focused pane.
    pub fn focused_session(&self) -> Option<SessionId> {
        self.panes.get(self.focused).map(|pane| pane.session)
    }

    /// The sessions its panes hold, in their order.
    pub fn sessions(&self) -> Vec<SessionId> {
        self.panes.iter().map(|pane| pane.session).collect()
    }

    /// Gives the focus to the pane at `index`.
    pub fn focus(&mut self, index: usize) {
        if index < self.panes.len() {
            self.focused = index;
        }
    }

    /// The pane the focus moves to, in the order the arrows walk.
    pub fn focus_neighbour(&self, direction: Direction) -> Option<usize> {
        let rows = self.rows();
        let (line, column) = rows.iter().enumerate().find_map(|(at, row)| {
            row.iter()
                .position(|pane| *pane == self.focused)
                .map(|c| (at, c))
        })?;
        let row = &rows[line];
        match direction {
            Direction::Left => {
                if column > 0 {
                    Some(row[column - 1])
                } else {
                    None
                }
            }
            Direction::Right => {
                if column + 1 < row.len() {
                    Some(row[column + 1])
                } else {
                    None
                }
            }
            Direction::Up => {
                if line > 0 {
                    let above = &rows[line - 1];
                    Some(above[column.min(above.len() - 1)])
                } else {
                    None
                }
            }
            Direction::Down => {
                if line + 1 < rows.len() {
                    let below = &rows[line + 1];
                    Some(below[column.min(below.len() - 1)])
                } else {
                    None
                }
            }
        }
    }

    /// The panes in rows, as a split lays them out: a pane that takes less than the whole height starts a new row.
    pub fn rows(&self) -> Vec<Vec<usize>> {
        let mut rows: Vec<Vec<usize>> = vec![Vec::new()];
        for (index, pane) in self.panes.iter().enumerate() {
            if pane.height < 10 && !rows.last().is_some_and(|row| row.is_empty()) {
                rows.push(Vec::new());
            }
            rows.last_mut().expect("a row").push(index);
        }
        rows
    }

    /// Splits the focused pane, and puts the session `session` in the new pane, which takes half the room and the focus.
    pub fn split(&mut self, vertical: bool, session: SessionId) -> Option<usize> {
        let at = self.focused;
        let old = self.panes.get_mut(at)?;
        let new = if vertical {
            let half = old.width / 2;
            old.width = half;
            Pane {
                session,
                width: half,
                height: old.height,
            }
        } else {
            let half = old.height / 2;
            old.height = half;
            Pane {
                session,
                width: old.width,
                height: half,
            }
        };
        self.panes.insert(at + 1, new);
        self.focused = at + 1;
        Some(at + 1)
    }

    /// Removes the pane at `index`; the tab is empty when the last one goes.
    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.panes.len() {
            return false;
        }
        self.panes.remove(index);
        if self.focused >= self.panes.len() {
            self.focused = self.panes.len().saturating_sub(1);
        }
        self.panes.is_empty()
    }
}

/// Where the focus moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(tab.panes[0].width, 5);
        assert_eq!(tab.panes[1].width, 5);
        assert_eq!(tab.panes[0].height, 10, "the height is untouched");
    }

    #[test]
    fn a_horizontal_split_leaves_two_panes_above_one_another() {
        let mut tab = Tab::new(0);
        tab.split(false, 0);
        assert_eq!(tab.panes.len(), 2);
        assert_eq!(tab.rows(), vec![vec![0], vec![1]], "two rows");
        assert_eq!(tab.panes[0].height, 5);
        assert_eq!(tab.panes[1].height, 5);
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
    fn a_tab_names_the_sessions_its_panes_hold() {
        let mut tab = Tab::new(1);
        assert_eq!(tab.sessions(), vec![1]);
        tab.split(true, 2);
        assert_eq!(tab.sessions(), vec![1, 2]);
        tab.split(false, 3);
        assert_eq!(tab.sessions(), vec![1, 2, 3]);
    }
}
