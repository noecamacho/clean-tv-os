//! An 8-bit glyph atlas with shelf packing.
//!
//! It starts small and grows up to [`MAX_WIDTH`] x [`MAX_HEIGHT`]. The CPU copy of the
//! bitmap is kept so glyphs can be added at any time and uploaded to the GPU in pieces.

pub const MAX_WIDTH: u32 = 1024;
pub const MAX_HEIGHT: u32 = 512;
const INITIAL_WIDTH: u32 = 512;
const INITIAL_HEIGHT: u32 = 256;
/// Empty pixels around every glyph so linear filtering never samples a neighbor.
const PADDING: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    next_x: u32,
}

/// What changed since the last upload.
#[derive(Debug, PartialEq, Eq)]
pub enum Update {
    /// The whole bitmap must be (re)created on the GPU.
    Full { width: u32, height: u32 },
    /// Only this region changed.
    Region(Slot),
}

#[derive(Debug)]
pub struct Atlas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    shelves: Vec<Shelf>,
    updates: Vec<Update>,
}

impl Default for Atlas {
    fn default() -> Self {
        Atlas::new()
    }
}

impl Atlas {
    pub fn new() -> Atlas {
        Atlas::with_size(INITIAL_WIDTH, INITIAL_HEIGHT)
    }

    fn with_size(width: u32, height: u32) -> Atlas {
        Atlas {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
            shelves: Vec::new(),
            updates: vec![Update::Full { width, height }],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Fraction of the bitmap area that is in use (including the shelves' wasted space).
    pub fn used_fraction(&self) -> f32 {
        let used: u32 = self.shelves.last().map_or(0, |s| s.y + s.height);
        used as f32 / self.height as f32
    }

    /// Takes the pending GPU updates, oldest first.
    pub fn take_updates(&mut self) -> Vec<Update> {
        std::mem::take(&mut self.updates)
    }

    /// Copies the pixels of `region`, row by row.
    pub fn region_pixels(&self, region: Slot) -> Vec<u8> {
        let mut out = Vec::with_capacity((region.width * region.height) as usize);
        for row in region.y..region.y + region.height {
            let start = (row * self.width + region.x) as usize;
            out.extend_from_slice(&self.pixels[start..start + region.width as usize]);
        }
        out
    }

    /// Finds room for a `width` x `height` bitmap and copies it in.
    /// Returns `None` when the atlas is full at its maximum size.
    pub fn insert(&mut self, width: u32, height: u32, bitmap: &[u8]) -> Option<Slot> {
        debug_assert_eq!(bitmap.len(), (width * height) as usize);
        let slot = loop {
            if let Some(slot) = self.find_slot(width, height) {
                break slot;
            }
            if !self.grow() {
                return None;
            }
        };
        for row in 0..height {
            let src = (row * width) as usize;
            let dst = ((slot.y + row) * self.width + slot.x) as usize;
            self.pixels[dst..dst + width as usize]
                .copy_from_slice(&bitmap[src..src + width as usize]);
        }
        // A pending full upload already covers this region.
        if !self
            .updates
            .iter()
            .any(|u| matches!(u, Update::Full { .. }))
        {
            self.updates.push(Update::Region(slot));
        }
        Some(slot)
    }

    fn find_slot(&mut self, width: u32, height: u32) -> Option<Slot> {
        let (w, h) = (width + PADDING, height + PADDING);
        if w > self.width || h > self.height {
            return None;
        }
        // Reuse a shelf that is tall enough without wasting too much height.
        let best = self
            .shelves
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.height >= h && s.height <= h + h / 2 + 2 && s.next_x + w <= self.width
            })
            .min_by_key(|(_, s)| s.height)
            .map(|(i, _)| i);
        if let Some(i) = best {
            let shelf = &mut self.shelves[i];
            let slot = Slot {
                x: shelf.next_x,
                y: shelf.y,
                width,
                height,
            };
            shelf.next_x += w;
            return Some(slot);
        }
        let y = self.shelves.last().map_or(0, |s| s.y + s.height);
        if y + h > self.height {
            return None;
        }
        self.shelves.push(Shelf {
            y,
            height: h,
            next_x: w,
        });
        Some(Slot {
            x: 0,
            y,
            width,
            height,
        })
    }

    /// Doubles the atlas until it reaches the maximum size, keeping its content.
    fn grow(&mut self) -> bool {
        if self.width >= MAX_WIDTH && self.height >= MAX_HEIGHT {
            return false;
        }
        let (new_width, new_height) = (
            (self.width * 2).min(MAX_WIDTH),
            (self.height * 2).min(MAX_HEIGHT),
        );
        let mut pixels = vec![0u8; (new_width * new_height) as usize];
        for row in 0..self.height {
            let src = (row * self.width) as usize;
            let dst = (row * new_width) as usize;
            pixels[dst..dst + self.width as usize]
                .copy_from_slice(&self.pixels[src..src + self.width as usize]);
        }
        self.width = new_width;
        self.height = new_height;
        self.pixels = pixels;
        // Texture coordinates change, so the GPU copy has to be recreated.
        self.updates = vec![Update::Full {
            width: new_width,
            height: new_height,
        }];
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(w: u32, h: u32, value: u8) -> Vec<u8> {
        vec![value; (w * h) as usize]
    }

    #[test]
    fn inserts_copy_pixels_and_do_not_overlap() {
        let mut atlas = Atlas::new();
        let a = atlas.insert(10, 12, &block(10, 12, 7)).unwrap();
        let b = atlas.insert(10, 12, &block(10, 12, 9)).unwrap();
        assert_eq!((a.x, a.y), (0, 0));
        assert!(b.x >= a.x + a.width + PADDING || b.y >= a.y + a.height + PADDING);
        assert_eq!(atlas.region_pixels(a), block(10, 12, 7));
        assert_eq!(atlas.region_pixels(b), block(10, 12, 9));
    }

    #[test]
    fn padding_stays_empty() {
        let mut atlas = Atlas::new();
        let a = atlas.insert(4, 4, &block(4, 4, 255)).unwrap();
        assert_eq!(atlas.pixels()[(a.x + a.width) as usize], 0);
        assert_eq!(
            atlas.pixels()[((a.y + a.height) * atlas.width()) as usize],
            0
        );
    }

    #[test]
    fn the_first_update_is_a_full_upload_then_regions() {
        let mut atlas = Atlas::new();
        assert_eq!(
            atlas.take_updates(),
            [Update::Full {
                width: 512,
                height: 256
            }]
        );
        let slot = atlas.insert(5, 5, &block(5, 5, 1)).unwrap();
        assert_eq!(atlas.take_updates(), [Update::Region(slot)]);
        assert!(atlas.take_updates().is_empty());
    }

    #[test]
    fn regions_are_folded_into_a_pending_full_upload() {
        let mut atlas = Atlas::new();
        atlas.insert(5, 5, &block(5, 5, 1)).unwrap();
        assert_eq!(atlas.take_updates().len(), 1);
    }

    #[test]
    fn grows_up_to_the_maximum_and_keeps_existing_glyphs() {
        let mut atlas = Atlas::new();
        let first = atlas.insert(30, 30, &block(30, 30, 200)).unwrap();
        atlas.take_updates();
        let mut inserted = 0;
        while atlas.insert(60, 60, &block(60, 60, 1)).is_some() {
            inserted += 1;
        }
        assert!(inserted > 40, "inserted only {inserted}");
        assert_eq!((atlas.width(), atlas.height()), (MAX_WIDTH, MAX_HEIGHT));
        assert_eq!(atlas.region_pixels(first), block(30, 30, 200));
        assert!(matches!(
            atlas.take_updates().as_slice(),
            [
                Update::Full {
                    width: 1024,
                    height: 512
                },
                ..
            ]
        ));
    }

    #[test]
    fn a_bitmap_larger_than_the_maximum_never_fits() {
        let mut atlas = Atlas::new();
        assert!(atlas.insert(2000, 10, &block(2000, 10, 1)).is_none());
        assert_eq!(atlas.width(), INITIAL_WIDTH.max(atlas.width()));
    }

    #[test]
    fn similar_heights_share_a_shelf() {
        let mut atlas = Atlas::new();
        let a = atlas.insert(10, 20, &block(10, 20, 1)).unwrap();
        let b = atlas.insert(10, 18, &block(10, 18, 1)).unwrap();
        assert_eq!(a.y, b.y);
    }
}
