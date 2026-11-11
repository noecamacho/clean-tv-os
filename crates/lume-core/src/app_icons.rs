//! What the app tiles and the dock show: one RGBA texture with two areas. The top has 16:9 cells,
//! one per app, for the tiles. The strip below it has square cells for the dock's icons, which
//! only the favorites use.
//!
//! The cells have the size a tile and a dock icon reach on screen at their largest (a focused tile,
//! a favorite at the peak of the Magnification) at the current scale, so no picture is enlarged
//! more than 1.05x (design-system and display-adaptation); a new scale gives a new [`AtlasLayout`],
//! and the texture is made again at that size.
//!
//! A cell holds the app's TV banner, which fills the whole tile, or, when the app has none, its
//! icon centered on a transparent background, so the tile color shows through. The platform
//! fetches the pixels and uploads them to the cell reserved here. Nothing is kept on the CPU
//! side, so everything goes away with the GPU context when lume goes dormant, and the cells are
//! filled again at the next wake.

use crate::render::Rect;
use std::collections::HashMap;

/// 16:9 cells per row and rows: room for 56 apps.
const COLUMNS: u32 = 4;
const ROWS: u32 = 14;
const CAPACITY: usize = (COLUMNS * ROWS) as usize;
/// Dock cells: more than the 12 favorites the dock can hold.
const DOCK_CAPACITY: usize = 16;
const _: () = assert!(DOCK_CAPACITY >= crate::home::config::MAX_FAVORITES);

/// The sizes of the texture and of its cells, in pixels, for one scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasLayout {
    /// A 16:9 tile cell.
    pub cell: [u32; 2],
    /// The side of a square dock cell.
    pub dock_cell: u32,
    /// The whole texture.
    pub width: u32,
    pub height: u32,
}

impl AtlasLayout {
    /// The layout for `scale` (device pixels per canvas unit), whose cells hold a tile of
    /// `tile_peak` canvas units wide and a dock icon of `dock_peak` at their largest. Sides are
    /// rounded up to a multiple of 4.
    pub fn new(scale: f32, tile_peak: f32, dock_peak: f32) -> AtlasLayout {
        let up_to_4 = |length: f32| ((length * scale / 4.0).ceil() as u32 * 4).max(4);
        let cell_width = up_to_4(tile_peak);
        let cell = [cell_width, (cell_width as f32 * 9.0 / 16.0).round() as u32];
        let dock_cell = up_to_4(dock_peak);
        let width = COLUMNS * cell[0];
        let dock_rows = (DOCK_CAPACITY as u32).div_ceil(Self::dock_columns(width, dock_cell));
        AtlasLayout {
            cell,
            dock_cell,
            width,
            height: ROWS * cell[1] + dock_rows * dock_cell,
        }
    }

    /// The layout at `scale` for the tiles and the dock as lume draws them.
    pub fn for_scale(scale: f32) -> AtlasLayout {
        let (tile_peak, dock_peak) = crate::app::image_peaks();
        AtlasLayout::new(scale, tile_peak, dock_peak)
    }

    fn dock_columns(width: u32, dock_cell: u32) -> u32 {
        (width / dock_cell).max(1)
    }

    /// Bytes of the texture (RGBA).
    pub fn bytes(&self) -> usize {
        (self.width * self.height * 4) as usize
    }
}

impl Default for AtlasLayout {
    fn default() -> AtlasLayout {
        AtlasLayout::for_scale(1.0)
    }
}

/// What a cell holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Art {
    /// The app's TV banner, rectangular: the shader rounds its corners at the tile's final size.
    /// It covers the tile.
    Banner,
    /// The app's icon, centered, over transparency.
    Icon,
}

#[derive(Default)]
pub struct AppIcons {
    layout: AtlasLayout,
    slots: HashMap<String, (usize, Art)>,
    free_slots: Vec<usize>,
    /// Who holds each dock cell.
    dock: [Option<String>; DOCK_CAPACITY],
}

impl AppIcons {
    /// Number of app tiles currently holding an atlas slot.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The sizes of the texture and its cells.
    pub fn layout(&self) -> AtlasLayout {
        self.layout
    }

    /// Takes the layout of a new scale. When it changes, every cell is forgotten (the texture is
    /// made again at the new size and the pictures fetched again at theirs); returns whether it did.
    pub fn set_layout(&mut self, layout: AtlasLayout) -> bool {
        if layout == self.layout {
            return false;
        }
        self.layout = layout;
        self.clear();
        true
    }

    /// Whether the app has a cell (or has no room for one).
    pub fn has(&self, package: &str) -> bool {
        self.slots.contains_key(package)
    }

    /// Whether there is still a free cell.
    pub fn has_room(&self) -> bool {
        self.slots.len() < CAPACITY
    }

    /// Reserves the cell of `package` and returns its pixel region `[x, y, width, height]`.
    /// `None` when the texture is full.
    pub fn reserve(&mut self, package: &str, art: Art) -> Option<[u32; 4]> {
        if let Some(&(slot, _)) = self.slots.get(package) {
            self.slots.insert(package.to_string(), (slot, art));
            return Some(self.region(slot));
        }
        if !self.has_room() {
            return None;
        }
        let slot = self.free_slots.pop().unwrap_or(self.slots.len());
        self.slots.insert(package.to_string(), (slot, art));
        Some(self.region(slot))
    }

    /// Removes an app from the atlas, making its slot available for reuse.
    pub fn remove(&mut self, package: &str) -> bool {
        if let Some((slot, _)) = self.slots.remove(package) {
            self.free_slots.push(slot);
            true
        } else {
            false
        }
    }

    /// The cell of `package`, by number.
    pub fn slot(&self, package: &str) -> Option<usize> {
        self.slots.get(package).map(|&(slot, _)| slot)
    }

    /// What the cell of `package` holds.
    pub fn art(&self, package: &str) -> Option<Art> {
        self.slots.get(package).map(|&(_, art)| art)
    }

    /// Where the cell is in the texture, in normalized coordinates.
    pub fn uv(&self, package: &str) -> Option<Rect> {
        Some(self.normalized(self.region(self.slots.get(package)?.0)))
    }

    /// Whether the app has a dock cell.
    pub fn has_dock(&self, package: &str) -> bool {
        self.dock_slot(package).is_some()
    }

    /// Reserves a dock cell for `package` (the same one if it already has it) and returns its
    /// pixel region `[x, y, width, height]`. `None` when all the dock cells are taken.
    pub fn reserve_dock(&mut self, package: &str) -> Option<[u32; 4]> {
        let slot = match self.dock_slot(package) {
            Some(slot) => slot,
            None => {
                let free = self.dock.iter().position(Option::is_none)?;
                self.dock[free] = Some(package.to_string());
                free
            }
        };
        Some(self.dock_region(slot))
    }

    /// Gives the dock cell of `package` back, for the next favorite to use.
    pub fn release_dock(&mut self, package: &str) {
        if let Some(slot) = self.dock_slot(package) {
            self.dock[slot] = None;
        }
    }

    /// Gives back the dock cell of every app that is not in `keep`.
    pub fn release_dock_except(&mut self, keep: &[String]) {
        for holder in &mut self.dock {
            if holder.as_ref().is_some_and(|p| !keep.contains(p)) {
                *holder = None;
            }
        }
    }

    /// Where the dock cell of `package` is in the texture, in normalized coordinates.
    pub fn dock_uv(&self, package: &str) -> Option<Rect> {
        Some(self.normalized(self.dock_region(self.dock_slot(package)?)))
    }

    /// Forgets every cell: the texture is gone.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_slots.clear();
        self.dock = Default::default();
    }

    /// The dock cell of `package`, by number.
    pub fn dock_slot(&self, package: &str) -> Option<usize> {
        self.dock
            .iter()
            .position(|holder| holder.as_deref() == Some(package))
    }

    /// A pixel region in normalized coordinates, half a pixel in on every side so linear filtering
    /// does not read the neighboring cell.
    fn normalized(&self, [x, y, w, h]: [u32; 4]) -> Rect {
        let (width, height) = (self.layout.width as f32, self.layout.height as f32);
        Rect::new(
            (x as f32 + 0.5) / width,
            (y as f32 + 0.5) / height,
            (w as f32 - 1.0) / width,
            (h as f32 - 1.0) / height,
        )
    }

    fn dock_region(&self, slot: usize) -> [u32; 4] {
        let side = self.layout.dock_cell;
        let columns = AtlasLayout::dock_columns(self.layout.width, side);
        let (column, row) = (slot as u32 % columns, slot as u32 / columns);
        [
            column * side,
            ROWS * self.layout.cell[1] + row * side,
            side,
            side,
        ]
    }

    fn region(&self, slot: usize) -> [u32; 4] {
        let [w, h] = self.layout.cell;
        let (column, row) = (slot as u32 % COLUMNS, slot as u32 / COLUMNS);
        [column * w, row * h, w, h]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell() -> [u32; 2] {
        AtlasLayout::default().cell
    }

    #[test]
    fn removed_slots_are_reused_before_hitting_capacity() {
        let mut icons = AppIcons::default();
        for i in 0..CAPACITY {
            let pkg = format!("com.pkg{i}");
            assert!(icons.reserve(&pkg, Art::Banner).is_some());
        }
        assert_eq!(icons.len(), CAPACITY);
        assert!(!icons.has_room());
        assert!(icons.reserve("com.overflow", Art::Banner).is_none());

        // Remove 2 packages
        assert!(icons.remove("com.pkg0"));
        assert!(icons.remove("com.pkg1"));
        assert_eq!(icons.len(), CAPACITY - 2);
        assert!(icons.has_room());

        // Insert 2 new packages
        let r1 = icons.reserve("com.new0", Art::Banner);
        let r2 = icons.reserve("com.new1", Art::Banner);
        assert!(r1.is_some());
        assert!(r2.is_some());
        assert_eq!(icons.len(), CAPACITY);
        assert_ne!(icons.slot("com.new0"), icons.slot("com.new1"));

        // Verify no collision with still-active packages
        let s0 = icons.slot("com.new0").unwrap();
        let s1 = icons.slot("com.new1").unwrap();
        for i in 2..CAPACITY {
            let pkg = format!("com.pkg{i}");
            let s = icons.slot(&pkg).unwrap();
            assert_ne!(s0, s);
            assert_ne!(s1, s);
        }
    }

    #[test]
    fn the_cells_have_the_size_of_the_largest_tile_and_dock_icon_at_each_scale() {
        // Home banner 212 wide (320 px at 1080p, the size Android TV banners are made at),
        // favorite 64 x 1.6 = 102.4 at the Magnification peak.
        let hd = AtlasLayout::for_scale(1.0);
        assert_eq!((hd.cell, hd.dock_cell), ([212, 119], 104));
        let full_hd = AtlasLayout::for_scale(1.5);
        assert_eq!((full_hd.cell, full_hd.dock_cell), ([320, 180], 156));
        assert_eq!((full_hd.width, full_hd.height), (1280, 14 * 180 + 2 * 156));
        // Smaller than the 1024x2304 (9.4 MB) texture it replaces at 720p, a little larger at 1080p.
        assert!(hd.bytes() < 1024 * 2304 * 4);
        assert!(full_hd.bytes() < 16 * 1024 * 1024);
    }

    #[test]
    fn no_picture_is_enlarged_more_than_the_focus_growth_or_reduced_below_half() {
        let (tile_peak, dock_peak) = crate::app::image_peaks();
        for scale in [1.0, 1.5] {
            let layout = AtlasLayout::for_scale(scale);
            let tile_rest = tile_peak * scale;
            let dock_rest = crate::app::dock_rest() * scale;
            assert!(tile_peak * scale / layout.cell[0] as f32 <= 1.0);
            assert!(dock_peak * scale / layout.dock_cell as f32 <= 1.05);
            assert!(tile_rest / layout.cell[0] as f32 >= 0.5);
            assert!(dock_rest / layout.dock_cell as f32 >= 0.5, "{scale}");
        }
    }

    #[test]
    fn a_new_scale_forgets_the_cells_and_the_same_one_keeps_them() {
        let mut icons = AppIcons::default();
        icons.reserve("a", Art::Banner);
        icons.reserve_dock("a");
        assert!(!icons.set_layout(AtlasLayout::for_scale(1.0)));
        assert!(icons.has("a") && icons.has_dock("a"));
        assert!(icons.set_layout(AtlasLayout::for_scale(1.5)));
        assert!(!icons.has("a") && !icons.has_dock("a"));
        assert_eq!(icons.reserve("a", Art::Banner), Some([0, 0, 320, 180]));
    }

    #[test]
    fn each_app_gets_its_own_cell_and_keeps_it() {
        let mut icons = AppIcons::default();
        let [w, h] = cell();
        let first = icons.reserve("a", Art::Banner).unwrap();
        let second = icons.reserve("b", Art::Icon).unwrap();
        assert_eq!(first, [0, 0, w, h]);
        assert_eq!(second, [w, 0, w, h]);
        assert_eq!(icons.reserve("a", Art::Banner), Some(first));
        assert!(icons.has("a") && !icons.has("c"));
        assert_eq!(icons.art("a"), Some(Art::Banner));
        assert_eq!(icons.art("b"), Some(Art::Icon));
    }

    #[test]
    fn cells_wrap_to_the_next_row() {
        let mut icons = AppIcons::default();
        let [w, h] = cell();
        for i in 0..COLUMNS {
            icons.reserve(&format!("app{i}"), Art::Icon);
        }
        assert_eq!(icons.reserve("next", Art::Icon), Some([0, h, w, h]));
    }

    #[test]
    fn a_full_texture_refuses_new_apps() {
        let mut icons = AppIcons::default();
        for i in 0..CAPACITY {
            assert!(icons.reserve(&format!("app{i}"), Art::Icon).is_some());
        }
        assert!(icons.reserve("one more", Art::Icon).is_none());
        assert!(!icons.has_room());
    }

    #[test]
    fn every_cell_fits_in_the_texture() {
        for scale in [1.0, 1.5] {
            let mut icons = AppIcons::default();
            icons.set_layout(AtlasLayout::for_scale(scale));
            let layout = icons.layout();
            for i in 0..CAPACITY {
                let [x, y, w, h] = icons.reserve(&format!("app{i}"), Art::Icon).unwrap();
                assert!(x + w <= layout.width && y + h <= layout.height);
            }
        }
    }

    #[test]
    fn uv_stays_inside_the_cell() {
        let mut icons = AppIcons::default();
        let layout = icons.layout();
        icons.reserve("a", Art::Banner);
        let uv = icons.uv("a").unwrap();
        assert!(uv.x > 0.0 && uv.right() < layout.cell[0] as f32 / layout.width as f32);
        assert!(uv.y > 0.0 && uv.bottom() < layout.cell[1] as f32 / layout.height as f32);
        assert!(icons.uv("missing").is_none());
    }

    #[test]
    fn dock_cells_are_reserved_found_and_released() {
        let mut icons = AppIcons::default();
        let layout = icons.layout();
        let (side, top) = (layout.dock_cell, ROWS * layout.cell[1]);
        let a = icons.reserve_dock("a").unwrap();
        let b = icons.reserve_dock("b").unwrap();
        assert_eq!(a, [0, top, side, side]);
        assert_eq!(b, [side, top, side, side]);
        assert_eq!(icons.reserve_dock("a"), Some(a), "the same cell again");
        assert!(icons.has_dock("a") && !icons.has_dock("c"));
        // A released cell goes to the next app.
        icons.release_dock("a");
        assert!(!icons.has_dock("a") && icons.dock_uv("a").is_none());
        assert_eq!(icons.reserve_dock("c"), Some(a));
        // The grid cells are separate from the dock cells.
        assert!(!icons.has("a") && icons.reserve("a", Art::Icon).is_some());
    }

    #[test]
    fn the_dock_holds_at_least_twelve_icons_and_then_refuses() {
        let mut icons = AppIcons::default();
        for i in 0..DOCK_CAPACITY {
            assert!(icons.reserve_dock(&format!("app{i}")).is_some());
        }
        assert!(icons.reserve_dock("one more").is_none());
        icons.release_dock("app3");
        assert!(icons.reserve_dock("one more").is_some());
    }

    #[test]
    fn dock_cells_fit_the_texture_and_never_overlap_each_other_or_the_grid() {
        for scale in [1.0, 1.5] {
            let mut icons = AppIcons::default();
            icons.set_layout(AtlasLayout::for_scale(scale));
            let layout = icons.layout();
            let mut regions = Vec::new();
            for i in 0..DOCK_CAPACITY {
                let [x, y, w, h] = icons.reserve_dock(&format!("app{i}")).unwrap();
                assert!(x + w <= layout.width && y + h <= layout.height);
                assert!(y >= ROWS * layout.cell[1]);
                regions.push((x, y, w, h));
            }
            for (i, a) in regions.iter().enumerate() {
                for b in &regions[i + 1..] {
                    assert!(
                        a.0 + a.2 <= b.0
                            || b.0 + b.2 <= a.0
                            || a.1 + a.3 <= b.1
                            || b.1 + b.3 <= a.1
                    );
                }
            }
        }
    }

    #[test]
    fn dock_uv_stays_inside_its_cell_without_bleeding_into_the_neighbors() {
        let mut icons = AppIcons::default();
        let layout = icons.layout();
        for i in 0..DOCK_CAPACITY {
            icons.reserve_dock(&format!("app{i}"));
        }
        for i in 0..DOCK_CAPACITY {
            let package = format!("app{i}");
            let [x, y, w, h] = icons.dock_region(icons.dock_slot(&package).unwrap());
            let uv = icons.dock_uv(&package).unwrap();
            let (aw, ah) = (layout.width as f32, layout.height as f32);
            // Half a texel in on every side: linear filtering stays inside the cell.
            assert!(uv.x * aw >= x as f32 + 0.49 && uv.right() * aw <= (x + w) as f32 - 0.49);
            assert!(uv.y * ah >= y as f32 + 0.49 && uv.bottom() * ah <= (y + h) as f32 - 0.49);
        }
    }

    #[test]
    fn clear_forgets_everything() {
        let mut icons = AppIcons::default();
        let [w, h] = cell();
        icons.reserve("a", Art::Icon);
        icons.reserve_dock("a");
        icons.clear();
        assert!(!icons.has("a") && !icons.has_dock("a"));
        assert_eq!(icons.reserve("b", Art::Icon), Some([0, 0, w, h]));
    }
}
