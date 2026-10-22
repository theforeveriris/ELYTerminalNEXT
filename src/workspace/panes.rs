//! The borders between panes: the one after a pane drags the widths of its row, the one under a row drags the heights of the rows, and a double press evens a row out again.

use gpui::{
    ClickEvent, Context, Div, Entity, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, ParentElement, Pixels, StatefulInteractiveElement, Styled, div, px,
};

use super::Workspace;

/// A border being dragged: the pane on its near side, which way it divides, where the pointer stood when the press came, and how far it has moved since, in percent.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Dragging {
    pub index: usize,
    pub side_by_side: bool,
    pub from: Pixels,
    pub share: i16,
}

impl Workspace {
    /// Begins a drag on the border after pane `index`, or under its row, at `along`.
    fn begin_drag(
        &mut self,
        index: usize,
        side_by_side: bool,
        along: Pixels,
        cx: &mut Context<Self>,
    ) {
        self.pane_drag = Some(Dragging {
            index,
            side_by_side,
            from: along,
            share: 0,
        });
        log::debug!("app: the border at pane {index} is dragged");
        cx.notify();
    }

    /// Moves the border to `along`, eight pixels to a percent, and stops at a pane's least.
    fn drag_to(&mut self, along: Pixels, cx: &mut Context<Self>) {
        let Some(drag) = self.pane_drag.as_mut() else {
            return;
        };
        let share = (f32::from(along - drag.from) / 8.0).round() as i16;
        if share == drag.share {
            return;
        }
        let step = share - drag.share;
        drag.share = share;
        let (index, side_by_side) = (drag.index, drag.side_by_side);
        let moved = self.tab_mut().is_some_and(|tab| match side_by_side {
            true => tab.widen(index, step),
            false => tab.heighten(index, step),
        });
        if !moved {
            log::debug!("app: the border stops at a pane's least");
        }
        cx.notify();
    }

    /// Lets the border rest where it was dragged to; the pane keeps the share.
    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(drag) = self.pane_drag.take() {
            log::info!("app: pane {} rests where it was dragged", drag.index);
            cx.notify();
        }
    }

    /// Gives the panes of `index`'s row equal widths and the rows equal heights, on a double press.
    pub(super) fn even_panes(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.tab_mut().is_some_and(|tab| tab.even(index)) {
            log::info!("app: the panes around {index} are even again");
            cx.notify();
        }
    }
}

/// The border after pane `index`, dragged sideways to change the widths of its row.
pub(super) fn wide(workspace: &Entity<Workspace>, index: usize) -> impl IntoElement + use<> {
    let (down, moved, up, click) = (
        workspace.clone(),
        workspace.clone(),
        workspace.clone(),
        workspace.clone(),
    );
    div()
        .id(("pane-border", index))
        .w(px(3.0))
        .h_full()
        .cursor_col_resize()
        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _, cx| {
            down.update(cx, |this, cx| {
                this.begin_drag(index, true, event.position.x, cx)
            })
        })
        .on_mouse_move(move |event: &MouseMoveEvent, _, cx| {
            if event.pressed_button == Some(MouseButton::Left) {
                moved.update(cx, |this, cx| this.drag_to(event.position.x, cx));
            }
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |this, cx| this.end_drag(cx))
        })
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.click_count() >= 2 {
                click.update(cx, |this, cx| this.even_panes(index, cx));
            }
        })
}

/// The border under the row that starts at pane `index`, dragged up or down to change the rows' heights.
pub(super) fn tall(workspace: &Entity<Workspace>, index: usize) -> impl IntoElement + use<> {
    let (down, moved, up, click) = (
        workspace.clone(),
        workspace.clone(),
        workspace.clone(),
        workspace.clone(),
    );
    div()
        .id(("row-border", index))
        .h(px(3.0))
        .w_full()
        .cursor_row_resize()
        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _, cx| {
            down.update(cx, |this, cx| {
                this.begin_drag(index, false, event.position.y, cx)
            })
        })
        .on_mouse_move(move |event: &MouseMoveEvent, _, cx| {
            if event.pressed_button == Some(MouseButton::Left) {
                moved.update(cx, |this, cx| this.drag_to(event.position.y, cx));
            }
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            up.update(cx, |this, cx| this.end_drag(cx))
        })
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.click_count() >= 2 {
                click.update(cx, |this, cx| this.even_panes(index, cx));
            }
        })
}

/// The panes of the shown tab, laid out as a grid: a row's height weighs it against the others, a pane's width weighs it against its row's, and the borders between them are the drags.
pub(super) fn grid(workspace: &Workspace, entity: &Entity<Workspace>) -> Div {
    let Some(tab) = workspace.tab() else {
        return div().size_full();
    };
    if let Some(id) = workspace.zoom
        && tab.holds(id)
        && let Some(session) = workspace.session(id)
    {
        return div().size_full().child(session.terminal.clone());
    }
    let rows = tab.rows();
    let mut out = div().flex().flex_col().size_full();
    let mut first_row = true;
    for row in rows {
        let height = tab
            .panes
            .get(row[0])
            .map(|pane| f32::from(pane.height))
            .unwrap_or(1.0);
        // TAB-15: the border under a row drags the heights of the rows above and below it.
        if !first_row {
            out = out.child(tall(entity, row[0]));
        }
        first_row = false;
        let mut line = div()
            .flex()
            .flex_row()
            .min_h_0()
            .flex_grow(height.max(1.0))
            .flex_basis(px(0.0));
        for (at, index) in row.iter().enumerate() {
            let Some(pane) = tab.panes.get(*index) else {
                continue;
            };
            let Some(session) = workspace.session(pane.session) else {
                continue;
            };
            let focused = tab.focused == *index;
            let terminal = session.terminal.clone();
            let mut cell = div()
                .min_w_0()
                .min_h_0()
                .flex_grow(f32::from(pane.width).max(1.0))
                .flex_basis(px(0.0))
                .child(terminal);
            if !focused {
                // A pane that is not focused is dimmed, so the focus is obvious.
                cell = cell.opacity(0.72);
            }
            if at + 1 < row.len() {
                cell = cell.border_r_1();
                line = line.child(cell);
                // TAB-15: the border toward the next pane drags the share between them.
                line = line.child(wide(entity, *index));
                continue;
            }
            line = line.child(cell);
        }
        out = out.child(line);
    }
    out
}

#[cfg(test)]
mod tests;
