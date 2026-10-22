//! Tabs and panes: the workspace's tree of terminals.

/// One pane of a tab: a session, and how much of the tab it takes.
pub struct Pane {
    /// The session it shows, by its id, which never changes.
    pub session: SessionId,
    /// How much of its row's width it takes; the panes of a row weigh against one another.
    pub width: u16,
    /// How much of the tab's height its row takes; rows weigh against one another.
    pub height: u16,
    /// Whether it begins a row of its own, which a split above or below it makes true.
    pub row: bool,
}

/// A session's id, which stays the same as the session moves and closes.
pub type SessionId = u64;

/// The least share a pane keeps, so a drag never leaves a pane to vanish.
pub const LEAST: u16 = 5;

impl Pane {
    /// A pane that takes the whole tab.
    pub fn whole(session: SessionId) -> Self {
        Self {
            session,
            width: 100,
            height: 100,
            row: true,
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

    /// Whether one of its panes holds `session`.
    pub fn holds(&self, session: SessionId) -> bool {
        self.panes.iter().any(|pane| pane.session == session)
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

    /// The panes in rows, as a split lays them out: a pane whose `row` says so begins a new row, and the panes of a row weigh their widths against one another.
    pub fn rows(&self) -> Vec<Vec<usize>> {
        let mut rows: Vec<Vec<usize>> = Vec::new();
        for (index, pane) in self.panes.iter().enumerate() {
            if pane.row || rows.is_empty() {
                rows.push(Vec::new());
            }
            rows.last_mut().expect("a row").push(index);
        }
        if rows.is_empty() {
            rows.push(Vec::new());
        }
        rows
    }

    /// Splits the focused pane, and puts the session `session` in the new pane, which takes half the room and the focus.
    pub fn split(&mut self, vertical: bool, session: SessionId) -> Option<usize> {
        let at = self.focused;
        let old = self.panes.get_mut(at)?;
        let new = if vertical {
            let half = (old.width / 2).max(LEAST);
            old.width = old.width.saturating_sub(half).max(LEAST);
            Pane {
                session,
                width: half,
                height: old.height,
                row: false,
            }
        } else {
            let half = (old.height / 2).max(LEAST);
            old.height = old.height.saturating_sub(half).max(LEAST);
            Pane {
                session,
                width: old.width,
                height: half,
                row: true,
            }
        };
        self.panes.insert(at + 1, new);
        self.focused = at + 1;
        Some(at + 1)
    }

    /// Moves `share` percent of the width from the pane before `index` to it, or back, keeping both at least `LEAST`; false at a row's edge.
    pub fn widen(&mut self, index: usize, share: i16) -> bool {
        let Some(row) = self.row_of(index) else {
            return false;
        };
        let at = row.iter().position(|pane| *pane == index).unwrap_or(0);
        if at == 0 {
            return false;
        }
        let left = row[at - 1];
        let right = row[at];
        let (gave, took) = (self.panes[left].width, self.panes[right].width);
        let give = share.clamp(
            -i16::try_from(took - LEAST).unwrap_or(i16::MAX),
            i16::try_from(gave - LEAST).unwrap_or(i16::MAX),
        );
        if give == 0 {
            return false;
        }
        self.panes[left].width = (gave as i16 - give) as u16;
        self.panes[right].width = (took as i16 + give) as u16;
        true
    }

    /// Moves `share` of the height from the row above `index` to its row, or back; false at the top row.
    pub fn heighten(&mut self, index: usize, share: i16) -> bool {
        let rows = self.rows();
        let Some(line) = rows.iter().position(|row| row.contains(&index)) else {
            return false;
        };
        if line == 0 {
            return false;
        }
        let above = self.panes[rows[line - 1][0]].height;
        let below = self.panes[rows[line][0]].height;
        let give = share.clamp(
            -i16::try_from(below - LEAST).unwrap_or(i16::MAX),
            i16::try_from(above - LEAST).unwrap_or(i16::MAX),
        );
        if give == 0 {
            return false;
        }
        for row in &rows[..line] {
            for pane in row {
                self.panes[*pane].height = (above as i16 - give) as u16;
            }
        }
        for row in &rows[line..] {
            for pane in row {
                self.panes[*pane].height = (below as i16 + give) as u16;
            }
        }
        true
    }

    /// Gives every pane of `index`'s row an equal width, and every row an equal height.
    pub fn even(&mut self, index: usize) -> bool {
        let Some(row) = self.row_of(index) else {
            return false;
        };
        let count = row.len() as u16;
        if count < 2 {
            return false;
        }
        let rows = self.rows();
        let each = (100 / rows.len() as u16).max(LEAST);
        for line in &rows {
            for pane in line {
                self.panes[*pane].height = each;
            }
        }
        for pane in &row {
            self.panes[*pane].width = (100 / count).max(LEAST);
        }
        true
    }

    /// The panes sharing `index`'s row.
    fn row_of(&self, index: usize) -> Option<Vec<usize>> {
        self.rows().into_iter().find(|row| row.contains(&index))
    }

    /// Exchanges the focused pane with its neighbour in `direction`, and moves the focus with it.
    pub fn swap(&mut self, direction: Direction) -> bool {
        let Some(neighbour) = self.focus_neighbour(direction) else {
            return false;
        };
        let focused = self.focused;
        self.panes.swap(focused, neighbour);
        self.focused = neighbour;
        true
    }

    /// Removes the pane at `index`; the tab is empty when the last one goes, and the pane that led a row hands that to the next.
    pub fn remove(&mut self, index: usize) -> bool {
        if index >= self.panes.len() {
            return false;
        }
        let led_row = self.panes[index].row;
        self.panes.remove(index);
        if led_row && let Some(next) = self.panes.get_mut(index) {
            if next.row {
                // The next pane already leads a row of its own; nothing to hand on.
            } else {
                next.row = true;
            }
        }
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
}
