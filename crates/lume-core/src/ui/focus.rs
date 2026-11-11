//! D-pad focus: exactly one element is focused, and arrow keys move it between rows.
//!
//! A screen describes its focusable elements as rows (a tab bar, a row of tiles, one line of
//! a grid, one editor line). Left and right move inside a row and never wrap. Up and down
//! move to the neighboring row and land on the element whose center is closest to where the
//! focus was horizontally, so grids keep their column and a short last row catches the focus
//! on its nearest item.

use crate::render::Rect;

/// Identifies a focusable element within a screen. Screens choose their own numbering.
pub type FocusId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Focusable {
    pub id: FocusId,
    /// Where the element is, in the screen's content coordinates.
    pub rect: Rect,
    /// Disabled elements cannot take the focus.
    pub enabled: bool,
}

impl Focusable {
    pub fn new(id: FocusId, rect: Rect) -> Focusable {
        Focusable {
            id,
            rect,
            enabled: true,
        }
    }

    pub fn disabled(id: FocusId, rect: Rect) -> Focusable {
        Focusable {
            id,
            rect,
            enabled: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub items: Vec<Focusable>,
    /// The element to land on when the focus enters this row moving up, instead of the one
    /// closest to the previous position (for example the active tab of the top bar).
    pub entry_up: Option<FocusId>,
    /// The element to land on when the focus enters this row moving down (for example the
    /// first tile of a grid when coming from the top bar).
    pub entry_down: Option<FocusId>,
}

impl Row {
    pub fn new(items: Vec<Focusable>) -> Row {
        Row {
            items,
            entry_up: None,
            entry_down: None,
        }
    }

    pub fn with_entry_up(mut self, id: FocusId) -> Row {
        self.entry_up = Some(id);
        self
    }

    pub fn with_entry_down(mut self, id: FocusId) -> Row {
        self.entry_down = Some(id);
        self
    }

    fn has_enabled(&self) -> bool {
        self.items.iter().any(|i| i.enabled)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Focus {
    rows: Vec<Row>,
    current: Option<(usize, usize)>,
    /// Horizontal center remembered across vertical moves.
    sticky_x: f32,
}

impl Focus {
    pub fn new() -> Focus {
        Focus::default()
    }

    /// Replaces the rows (after a layout change). The focus stays on the element with the same
    /// id when it still exists and is enabled, otherwise it goes to the first enabled element.
    pub fn set_rows(&mut self, rows: Vec<Row>) {
        let keep = self.current_id();
        let position = self.current;
        self.rows = rows;
        self.current = None;
        if let Some(id) = keep {
            self.focus(id);
        }
        if self.current.is_none() {
            // The element is gone: stay near where it was, in the same row and column.
            if let Some((row, col)) = position {
                self.focus_near(row, col);
            }
        }
        if self.current.is_none() {
            self.focus_first();
        }
    }

    /// Focuses the enabled element closest to column `col` of row `row`, if that row exists.
    fn focus_near(&mut self, row: usize, col: usize) {
        let Some(items) = self.rows.get(row).map(|r| &r.items) else {
            return;
        };
        let best = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.enabled)
            .min_by_key(|(c, _)| c.abs_diff(col))
            .map(|(c, _)| c);
        if let Some(c) = best {
            self.current = Some((row, c));
            self.sticky_x = items[c].rect.center().0;
        }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn current(&self) -> Option<&Focusable> {
        let (row, col) = self.current?;
        self.rows.get(row)?.items.get(col)
    }

    pub fn current_id(&self) -> Option<FocusId> {
        self.current().map(|i| i.id)
    }

    pub fn is_focused(&self, id: FocusId) -> bool {
        self.current_id() == Some(id)
    }

    /// Index of the row that has the focus.
    pub fn current_row(&self) -> Option<usize> {
        self.current.map(|(row, _)| row)
    }

    /// Focuses the element `id`. Returns `false` when it does not exist or is disabled.
    pub fn focus(&mut self, id: FocusId) -> bool {
        for (r, row) in self.rows.iter().enumerate() {
            if let Some(c) = row.items.iter().position(|i| i.id == id && i.enabled) {
                self.current = Some((r, c));
                self.sticky_x = row.items[c].rect.center().0;
                return true;
            }
        }
        false
    }

    /// Focuses the first enabled element of the first row that has one.
    pub fn focus_first(&mut self) -> bool {
        for (r, row) in self.rows.iter().enumerate() {
            if let Some(c) = row.items.iter().position(|i| i.enabled) {
                self.current = Some((r, c));
                self.sticky_x = row.items[c].rect.center().0;
                return true;
            }
        }
        self.current = None;
        false
    }

    /// Moves the focus one step. Returns `true` when it moved.
    pub fn move_focus(&mut self, direction: Direction) -> bool {
        let Some((row, col)) = self.current else {
            return false;
        };
        match direction {
            Direction::Left | Direction::Right => {
                let items = &self.rows[row].items;
                let candidates: Box<dyn Iterator<Item = usize>> = if direction == Direction::Left {
                    Box::new((0..col).rev())
                } else {
                    Box::new(col + 1..items.len())
                };
                for c in candidates {
                    if items[c].enabled {
                        self.current = Some((row, c));
                        self.sticky_x = items[c].rect.center().0;
                        return true;
                    }
                }
                false
            }
            Direction::Up | Direction::Down => {
                let candidates: Box<dyn Iterator<Item = usize>> = if direction == Direction::Up {
                    Box::new((0..row).rev())
                } else {
                    Box::new(row + 1..self.rows.len())
                };
                for r in candidates {
                    if !self.rows[r].has_enabled() {
                        continue;
                    }
                    let c = self.landing_column(r, direction);
                    self.current = Some((r, c));
                    // The sticky position survives vertical moves, so a column is kept
                    // through rows of different lengths.
                    return true;
                }
                false
            }
        }
    }

    /// The column of row `r` to land on when moving there vertically.
    fn landing_column(&self, r: usize, direction: Direction) -> usize {
        let row = &self.rows[r];
        let entry = if direction == Direction::Up {
            row.entry_up
        } else {
            row.entry_down
        };
        if let Some(entry) = entry {
            if let Some(c) = row.items.iter().position(|i| i.id == entry && i.enabled) {
                return c;
            }
        }
        row.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.enabled)
            .min_by(|(_, a), (_, b)| {
                let (da, db) = (
                    (a.rect.center().0 - self.sticky_x).abs(),
                    (b.rect.center().0 - self.sticky_x).abs(),
                );
                da.total_cmp(&db)
            })
            .map(|(c, _)| c)
            .expect("the row has an enabled item")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: FocusId, column: u32, row: u32) -> Focusable {
        // 5-column grid of 200x100 cells.
        Focusable::new(
            id,
            Rect::new(column as f32 * 220.0, row as f32 * 140.0, 200.0, 100.0),
        )
    }

    /// Apps grid: 5 columns (the focus code does not depend on the real count), `count` items, ids 100.. in reading order.
    fn grid(count: u32) -> Vec<Row> {
        (0..count.div_ceil(5))
            .map(|r| {
                Row::new(
                    (0..5)
                        .filter(|c| r * 5 + c < count)
                        .map(|c| item(100 + r * 5 + c, c, r + 1))
                        .collect(),
                )
            })
            .collect()
    }

    fn with_topbar(mut rows: Vec<Row>) -> Vec<Row> {
        let tabs = vec![item(1, 0, 0), item(2, 1, 0), item(3, 4, 0)];
        rows.insert(0, Row::new(tabs).with_entry_up(2));
        rows
    }

    fn focus_on(rows: Vec<Row>, id: FocusId) -> Focus {
        let mut focus = Focus::new();
        focus.set_rows(rows);
        assert!(focus.focus(id));
        focus
    }

    #[test]
    fn starts_on_the_first_enabled_element() {
        let mut focus = Focus::new();
        assert_eq!(focus.current_id(), None);
        focus.set_rows(vec![Row::new(vec![
            Focusable::disabled(1, Rect::default()),
            item(2, 1, 0),
        ])]);
        assert_eq!(focus.current_id(), Some(2));
    }

    #[test]
    fn exactly_one_element_is_focused() {
        let focus = focus_on(grid(7), 103);
        let focused = (100..107).filter(|id| focus.is_focused(*id)).count();
        assert_eq!(focused, 1);
    }

    #[test]
    fn left_and_right_move_inside_the_row_and_do_not_wrap() {
        let mut focus = focus_on(grid(12), 103);
        assert!(focus.move_focus(Direction::Right));
        assert_eq!(focus.current_id(), Some(104));
        // 5th column: Right stays put and does not jump to the next row.
        assert!(!focus.move_focus(Direction::Right));
        assert_eq!(focus.current_id(), Some(104));
        let mut focus = focus_on(grid(12), 105);
        assert!(!focus.move_focus(Direction::Left));
        assert_eq!(focus.current_id(), Some(105));
    }

    #[test]
    fn vertical_moves_keep_the_column() {
        let mut focus = focus_on(grid(15), 102);
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(focus.current_id(), Some(107));
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(focus.current_id(), Some(112));
        assert!(!focus.move_focus(Direction::Down), "no row below");
        assert!(focus.move_focus(Direction::Up));
        assert_eq!(focus.current_id(), Some(107));
    }

    #[test]
    fn a_short_last_row_catches_the_focus_on_its_nearest_item() {
        // 17 items: the last row has only 2 items. From column 4 (index 3), Down lands on the last one.
        let mut focus = focus_on(grid(17), 113);
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(
            focus.current_id(),
            Some(116),
            "the last item of the last row"
        );
        // Going back up returns to the column the focus came from, not the short row's column.
        assert!(focus.move_focus(Direction::Up));
        assert_eq!(focus.current_id(), Some(113));
    }

    #[test]
    fn the_sticky_column_survives_a_short_row() {
        // Moving down through a short row and on: the original column is restored.
        let mut rows = grid(10);
        rows.insert(1, Row::new(vec![item(200, 0, 2)]));
        let mut focus = focus_on(rows, 103);
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(focus.current_id(), Some(200));
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(
            focus.current_id(),
            Some(108),
            "the column of the start (index 3) is back"
        );
    }

    #[test]
    fn up_from_the_first_row_goes_to_the_active_tab() {
        let mut focus = focus_on(with_topbar(grid(10)), 104);
        assert!(focus.move_focus(Direction::Up));
        assert_eq!(
            focus.current_id(),
            Some(2),
            "the active tab, not the closest element"
        );
        assert!(!focus.move_focus(Direction::Up));
        // Moving between the tabs and the gear.
        assert!(focus.move_focus(Direction::Right));
        assert_eq!(focus.current_id(), Some(3));
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(
            focus.current_id(),
            Some(104),
            "back below, in the closest column"
        );
    }

    #[test]
    fn tabs_move_with_left_and_right() {
        let mut focus = focus_on(with_topbar(grid(5)), 1);
        assert!(focus.move_focus(Direction::Right));
        assert_eq!(focus.current_id(), Some(2));
        assert!(!focus.move_focus(Direction::Left) || focus.current_id() == Some(1));
    }

    #[test]
    fn disabled_elements_are_skipped() {
        let rows = vec![
            Row::new(vec![
                item(1, 0, 0),
                Focusable::disabled(2, Rect::new(220.0, 0.0, 200.0, 100.0)),
                item(3, 2, 0),
            ]),
            Row::new(vec![Focusable::disabled(
                4,
                Rect::new(0.0, 140.0, 200.0, 100.0),
            )]),
            Row::new(vec![item(5, 0, 2)]),
        ];
        let mut focus = focus_on(rows, 1);
        assert!(focus.move_focus(Direction::Right));
        assert_eq!(
            focus.current_id(),
            Some(3),
            "the disabled element in between is skipped"
        );
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(
            focus.current_id(),
            Some(5),
            "a row with nothing enabled is skipped"
        );
        assert!(!focus.focus(2) && !focus.focus(4) && !focus.focus(99));
    }

    #[test]
    fn the_focus_survives_a_layout_change() {
        let mut focus = focus_on(grid(10), 107);
        // Same ids, shifted positions.
        let mut rows = grid(10);
        rows[1].items[2].rect.x += 1000.0;
        focus.set_rows(rows);
        assert_eq!(focus.current_id(), Some(107));
        // The focused element disappears and its row is gone too: the first element takes over.
        focus.set_rows(grid(4));
        assert_eq!(focus.current_id(), Some(100));
        focus.set_rows(vec![]);
        assert_eq!(focus.current_id(), None);
        assert!(!focus.move_focus(Direction::Down));
    }

    #[test]
    fn a_vanished_element_hands_the_focus_to_its_neighbor() {
        // The focus is on the 4th tile of a row that shrinks to 2 tiles: it lands on the last one.
        let mut focus = focus_on(grid(10), 108);
        let mut rows = grid(10);
        rows[1].items.truncate(2);
        focus.set_rows(rows);
        assert_eq!(
            focus.current_id(),
            Some(106),
            "the nearest tile of the same row"
        );
        // The row itself shrinks to nothing enabled: the first element of the screen.
        let mut rows = grid(10);
        rows[1].items.clear();
        focus.set_rows(rows);
        assert_eq!(focus.current_id(), Some(100));
    }

    #[test]
    fn editor_style_rows_use_the_horizontal_position() {
        // Two lines: [up, down, hide] on the left and chips on the right; heights differ.
        let left = |id, x| Focusable::new(id, Rect::new(x, 0.0, 48.0, 48.0));
        let rows = vec![
            Row::new(vec![
                left(1, 80.0),
                left(2, 136.0),
                left(3, 192.0),
                Focusable::new(4, Rect::new(900.0, 0.0, 60.0, 33.0)),
            ]),
            Row::new(vec![
                Focusable::new(5, Rect::new(80.0, 100.0, 48.0, 48.0)),
                Focusable::new(6, Rect::new(136.0, 100.0, 48.0, 48.0)),
                Focusable::new(7, Rect::new(192.0, 100.0, 48.0, 48.0)),
                Focusable::new(8, Rect::new(700.0, 100.0, 70.0, 33.0)),
                Focusable::new(9, Rect::new(800.0, 100.0, 70.0, 33.0)),
            ]),
        ];
        let mut focus = focus_on(rows.clone(), 2);
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(focus.current_id(), Some(6));
        let mut focus = focus_on(rows, 4);
        assert!(focus.move_focus(Direction::Down));
        assert_eq!(focus.current_id(), Some(9), "the chip nearest to x=930");
    }
}
