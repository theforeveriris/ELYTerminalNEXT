//! Tabs and panes: the workspace's tree of terminals.

/// One pane of a tab: a session, and how much of the tab it takes.
#[derive(Clone, Copy)]
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
#[derive(Clone)]
pub struct Tab {
    /// Its panes, in their order.
    pub panes: Vec<Pane>,
    /// The pane that has the focus, as an index into `panes`.
    pub focused: usize,
    /// The name the user gave it; empty leaves the title to the focused pane.
    pub name: String,
    /// Whether it stays where it is: a pinned tab is never moved by a reorder, and sits before the rest.
    pub pinned: bool,
    /// Whether its panes are folded away in the column, leaving only the tab's own row.
    pub collapsed: bool,
}

impl Tab {
    /// A tab with one pane, which has the focus.
    pub fn new(session: SessionId) -> Self {
        Self {
            panes: vec![Pane::whole(session)],
            focused: 0,
            name: String::new(),
            pinned: false,
            collapsed: false,
        }
    }

    /// Moves this tab one place up (`-1`) or down (`1`) among `tabs`, keeping the pinned ones first: a pinned tab does not move, and an unpinned one does not cross one.
    pub fn move_by(tabs: &mut [Tab], at: usize, by: isize) -> bool {
        let Some(tab) = tabs.get(at) else {
            return false;
        };
        if tab.pinned {
            return false;
        }
        let Some(to) = at.checked_add_signed(by).filter(|to| *to < tabs.len()) else {
            return false;
        };
        if tabs[to].pinned {
            return false;
        }
        tabs.swap(at, to);
        true
    }

    /// Puts `at` where its pin says it belongs: pinned tabs first, in the order they were pinned, and the rest after them.
    pub fn pin(tabs: &mut Vec<Tab>, at: usize, pinned: bool) -> Option<usize> {
        let mut tab = tabs.get(at).cloned()?;
        tab.pinned = pinned;
        tabs.remove(at);
        let to = match pinned {
            true => tabs.iter().take_while(|other| other.pinned).count(),
            false => tabs
                .iter()
                .position(|other| !other.pinned)
                .unwrap_or(tabs.len()),
        };
        tabs.insert(to, tab);
        Some(to)
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
mod tests;
