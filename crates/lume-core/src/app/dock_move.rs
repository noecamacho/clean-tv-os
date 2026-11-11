//! Moving a favorite along the dock, as "Move" in its context menu starts it: the neighbors make
//! room for an arrow on each side, Left and Right carry the favorite, OK keeps the new place and
//! Back puts it back where it was. The order is written only when it is kept.

use super::{ids, Effect, Key, LumeApp, PendingFocus};
use crate::ui::focus::FocusId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DockMove {
    pub package: String,
    /// The order before the move, for Back.
    original: Vec<String>,
}

impl LumeApp {
    /// The favorite being moved, if any (for tests).
    #[doc(hidden)]
    pub fn moving_favorite(&self) -> Option<&str> {
        self.dock_move.as_ref().map(|m| m.package.as_str())
    }

    pub(super) fn start_dock_move(&mut self, package: &str) {
        self.dock_move = Some(DockMove {
            package: package.to_string(),
            original: self.config.favorites.clone(),
        });
        self.animate_move_room(true);
    }

    /// A key while a favorite is being moved: the focus stays on it.
    pub(super) fn dock_move_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(state) = self.dock_move.as_ref() else {
            return vec![];
        };
        let Some(index) = self
            .config
            .favorites
            .iter()
            .position(|p| *p == state.package)
        else {
            self.end_dock_move_quietly();
            return vec![];
        };
        match key {
            Key::Left | Key::Right => {
                let count = self.config.favorites.len();
                let to = match key {
                    Key::Left => index.checked_sub(1),
                    _ => Some(index + 1).filter(|&to| to < count),
                };
                if let Some(to) = to {
                    self.begin_dock_transition(Some(to));
                    self.config.favorites.swap(index, to);
                    self.pending_focus = Some(PendingFocus::Id(favorite_id(to)));
                }
                vec![]
            }
            Key::Ok => {
                let kept = self.dock_move.take().map(|m| m.original)
                    != Some(self.config.favorites.clone());
                self.animate_move_room(false);
                if kept {
                    vec![Effect::SaveConfig]
                } else {
                    vec![]
                }
            }
            Key::Back | Key::Home => self.cancel_dock_move(),
            Key::Up | Key::Down | Key::Menu => vec![],
        }
    }

    /// Puts the favorites back in the order they had before the move.
    fn cancel_dock_move(&mut self) -> Vec<Effect> {
        let Some(state) = self.dock_move.take() else {
            return vec![];
        };
        self.animate_move_room(false);
        if state.original == self.config.favorites {
            return vec![];
        }
        let original: Vec<String> = state
            .original
            .into_iter()
            .filter(|p| self.catalog.contains(p))
            .collect();
        let back = original.iter().position(|p| *p == state.package);
        self.begin_dock_transition(back);
        self.config.favorites = original;
        if let Some(index) = back {
            self.pending_focus = Some(PendingFocus::Id(favorite_id(index)));
        }
        // Written although it is the old order: an uninstall may have saved the moved one meanwhile.
        vec![Effect::SaveConfig]
    }

    /// Leaves the move when the screen changes under it (Home, a return from an app), with the
    /// favorites in their old order and without a slide.
    pub(super) fn end_dock_move_quietly(&mut self) {
        if let Some(state) = self.dock_move.take() {
            self.config.favorites = state
                .original
                .into_iter()
                .filter(|p| self.catalog.contains(p))
                .collect();
            self.anim.set(super::Anim::DockRoom, 0.0);
        }
    }

    /// The catalog changed: a move whose favorite was uninstalled ends.
    pub(super) fn check_dock_move(&mut self) {
        let gone = self
            .dock_move
            .as_ref()
            .is_some_and(|m| !self.config.favorites.contains(&m.package));
        if gone {
            self.end_dock_move_quietly();
        }
    }
}

fn favorite_id(index: usize) -> FocusId {
    ids::FAVORITE_BASE + index as FocusId
}
