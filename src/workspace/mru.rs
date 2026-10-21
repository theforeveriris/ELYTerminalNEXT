//! The sessions in the order they were last shown, for Ctrl-Tab.

use super::layout::SessionId;

/// Sessions by when they were last shown, newest first.
#[derive(Debug, Default)]
pub struct Mru {
    order: Vec<SessionId>,
}

impl Mru {
    /// Notes that `id` is shown now, and moves it to the front.
    pub fn touch(&mut self, id: SessionId) {
        self.order.retain(|other| *other != id);
        self.order.insert(0, id);
    }

    /// Forgets a session that closed.
    pub fn forget(&mut self, id: SessionId) {
        self.order.retain(|other| *other != id);
    }

    /// The session to show next from `active`: the newest or the oldest other one.
    pub fn next(&self, active: SessionId, forward: bool) -> Option<SessionId> {
        let mut others = self.order.iter().copied().filter(|id| *id != active);
        if forward {
            others.next()
        } else {
            others.next_back()
        }
    }
}

#[cfg(test)]
mod tests;
