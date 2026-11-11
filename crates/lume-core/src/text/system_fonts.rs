//! Fallback to the fonts installed on the device.
//!
//! On Android the candidates come from `/system/etc/fonts.xml`, in the order the
//! system lists them. A font file is opened with `mmap` only when needed: first its
//! `cmap` is consulted straight from the mapping, and only a file that has the glyph
//! is turned into a rasterizable font. Everything is dropped by
//! [`FallbackFonts::release`], so the mapped pages and glyph data cost nothing while
//! lume is in the background.

use super::FallbackFonts;
use fontdue::{Font, FontSettings};
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// One font file (and collection index) that may be used as a fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontCandidate {
    pub path: PathBuf,
    /// Face index inside a `.ttc` collection; 0 for single fonts.
    pub index: u32,
}

/// Reads the regular-weight fonts of `fonts.xml`, in document order and without duplicates.
/// Relative file names are resolved against `font_dir`.
pub fn parse_fonts_xml(xml: &str, font_dir: &Path) -> Vec<FontCandidate> {
    let mut candidates: Vec<FontCandidate> = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<font") {
        rest = &rest[start + "<font".len()..];
        // `<font ...>` only (not `<fontx` or `<fonts`).
        if !rest.starts_with(|c: char| c.is_whitespace() || c == '>') {
            continue;
        }
        let Some(tag_end) = rest.find('>') else { break };
        let attributes = &rest[..tag_end];
        let after = &rest[tag_end + 1..];
        let Some(close) = after.find("</font>") else {
            break;
        };
        let file = after[..close].trim();
        rest = &after[close..];

        let weight: u32 = attribute(attributes, "weight")
            .and_then(|w| w.parse().ok())
            .unwrap_or(400);
        let style = attribute(attributes, "style").unwrap_or("normal");
        if weight != 400 || style != "normal" || file.is_empty() {
            continue;
        }
        let index = attribute(attributes, "index")
            .and_then(|i| i.parse().ok())
            .unwrap_or(0);
        let candidate = FontCandidate {
            path: font_dir.join(file),
            index,
        };
        if !candidates.contains(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

fn attribute<'a>(attributes: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let start = attributes.find(&needle)? + needle.len();
    let end = attributes[start..].find('"')?;
    Some(&attributes[start..start + end])
}

pub struct SystemFonts {
    candidates: Vec<FontCandidate>,
    /// Memory maps, opened lazily. `None` after a failed attempt.
    maps: Vec<Option<Option<Mmap>>>,
    /// Cached codepoint coverage intervals `(start, end)` per candidate.
    coverage: Vec<Option<Vec<(u32, u32)>>>,
    /// Candidate indexes whose font face could not be parsed.
    invalid: Vec<bool>,
    loaded: Vec<Font>,
    /// Candidate index -> index in `loaded`.
    loaded_for: HashMap<usize, usize>,
    /// Characters already looked up (hits and misses).
    results: HashMap<char, Option<usize>>,
}

impl SystemFonts {
    pub fn new(candidates: Vec<FontCandidate>) -> SystemFonts {
        let maps = candidates.iter().map(|_| None).collect();
        let coverage = candidates.iter().map(|_| None).collect();
        let invalid = vec![false; candidates.len()];
        SystemFonts {
            candidates,
            maps,
            coverage,
            invalid,
            loaded: Vec::new(),
            loaded_for: HashMap::new(),
            results: HashMap::new(),
        }
    }

    /// Reads a `fonts.xml` file; font names are relative to `font_dir`.
    pub fn from_fonts_xml(xml_path: &Path, font_dir: &Path) -> std::io::Result<SystemFonts> {
        let xml = std::fs::read_to_string(xml_path)?;
        Ok(SystemFonts::new(parse_fonts_xml(&xml, font_dir)))
    }

    /// The Android system font setup.
    pub fn android() -> std::io::Result<SystemFonts> {
        SystemFonts::from_fonts_xml(
            Path::new("/system/etc/fonts.xml"),
            Path::new("/system/fonts"),
        )
    }

    /// Number of fonts currently turned into rasterizable fonts.
    pub fn loaded_fonts(&self) -> usize {
        self.loaded.len()
    }

    /// Number of font files currently mapped.
    pub fn mapped_files(&self) -> usize {
        self.maps.iter().flatten().flatten().count()
    }

    fn map(&mut self, candidate: usize) -> Option<&Mmap> {
        if self.maps[candidate].is_none() {
            let map = File::open(&self.candidates[candidate].path)
                .ok()
                .and_then(|file| {
                    // SAFETY: system font files are read-only and are not modified while mapped.
                    unsafe { Mmap::map(&file) }.ok()
                });
            self.maps[candidate] = Some(map);
        }
        self.maps[candidate].as_ref().and_then(Option::as_ref)
    }

    /// Whether the candidate's `cmap` has `c`, read straight from the mapping.
    fn covers(&mut self, candidate: usize, c: char) -> bool {
        if self.invalid[candidate] {
            return false;
        }
        if let Some(intervals) = &self.coverage[candidate] {
            let cp = c as u32;
            return intervals
                .binary_search_by(|&(start, end)| {
                    if cp < start {
                        std::cmp::Ordering::Greater
                    } else if cp > end {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
                .is_ok();
        }

        let index = self.candidates[candidate].index;
        let Some(map) = self.map(candidate) else {
            return false;
        };
        let Ok(face) = ttf_parser::Face::parse(map, index) else {
            self.invalid[candidate] = true;
            return false;
        };

        let mut cps = Vec::new();
        if let Some(cmap) = face.tables().cmap {
            for sub in cmap.subtables {
                if sub.is_unicode() {
                    sub.codepoints(|cp| cps.push(cp));
                }
            }
        }
        cps.sort_unstable();
        cps.dedup();

        let mut intervals: Vec<(u32, u32)> = Vec::new();
        for cp in cps {
            if let Some(last) = intervals.last_mut() {
                if last.1 + 1 == cp {
                    last.1 = cp;
                    continue;
                }
            }
            intervals.push((cp, cp));
        }

        let cp = c as u32;
        let covered = intervals
            .binary_search_by(|&(start, end)| {
                if cp < start {
                    std::cmp::Ordering::Greater
                } else if cp > end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok();

        self.coverage[candidate] = Some(intervals);
        covered
    }

    fn load(&mut self, candidate: usize) -> Option<usize> {
        if let Some(&index) = self.loaded_for.get(&candidate) {
            return Some(index);
        }
        let collection_index = self.candidates[candidate].index;
        let map = self.map(candidate)?;
        let settings = FontSettings {
            collection_index,
            ..FontSettings::default()
        };
        let font = Font::from_bytes(&map[..], settings).ok()?;
        self.loaded.push(font);
        self.loaded_for.insert(candidate, self.loaded.len() - 1);
        Some(self.loaded.len() - 1)
    }
}

impl FallbackFonts for SystemFonts {
    fn find(&mut self, c: char) -> Option<usize> {
        if let Some(result) = self.results.get(&c) {
            return *result;
        }
        let mut found = None;
        for candidate in 0..self.candidates.len() {
            let already_loaded = self.loaded_for.get(&candidate).copied();
            let has_glyph = match already_loaded {
                Some(index) => self.loaded[index].lookup_glyph_index(c) != 0,
                None => self.covers(candidate, c),
            };
            if has_glyph {
                found = self.load(candidate);
                if found.is_some() {
                    break;
                }
            }
        }
        self.results.insert(c, found);
        found
    }

    fn font(&self, index: usize) -> &Font {
        &self.loaded[index]
    }

    fn release(&mut self) {
        self.loaded.clear();
        self.loaded_for.clear();
        self.results.clear();
        self.maps.fill_with(|| None);
        self.coverage.fill(None);
        self.invalid.fill(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{Cmd, DisplayList};
    use crate::text::{TextEngine, TextStyle};
    use crate::theme::Color;

    const FONTS_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<familyset version="23">
    <family name="sans-serif">
        <font weight="400" style="normal">Roboto-Regular.ttf</font>
        <font weight="400" style="italic">Roboto-Italic.ttf</font>
        <font weight="700" style="normal">Roboto-Bold.ttf</font>
    </family>
    <family lang="und-Cyrl">
        <font weight="400" style="normal">NotoSans-Regular.ttf</font>
    </family>
    <family lang="ja">
        <font weight="400" style="normal" index="2" postScriptName="NotoSansCJKjp-Regular">NotoSansCJK-Regular.ttc</font>
    </family>
    <family lang="ko">
        <font weight="400" style="normal" index="1">NotoSansCJK-Regular.ttc</font>
        <font weight="400" style="normal" index="2">NotoSansCJK-Regular.ttc</font>
    </family>
    <family name="serif"><font weight="400" style="normal">Roboto-Regular.ttf</font></family>
</familyset>"#;

    fn test_font(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fonts")
            .join(name)
    }

    fn candidates() -> Vec<FontCandidate> {
        ["test-cyrillic.ttf", "test-cjk.ttf"]
            .iter()
            .map(|n| FontCandidate {
                path: test_font(n),
                index: 0,
            })
            .collect()
    }

    #[test]
    fn fonts_xml_yields_regular_fonts_in_order_without_duplicates() {
        let found = parse_fonts_xml(FONTS_XML, Path::new("/system/fonts"));
        let names: Vec<_> = found
            .iter()
            .map(|c| {
                (
                    c.path.file_name().unwrap().to_str().unwrap().to_string(),
                    c.index,
                )
            })
            .collect();
        assert_eq!(
            names,
            [
                ("Roboto-Regular.ttf".to_string(), 0),
                ("NotoSans-Regular.ttf".to_string(), 0),
                ("NotoSansCJK-Regular.ttc".to_string(), 2),
                ("NotoSansCJK-Regular.ttc".to_string(), 1),
            ]
        );
        assert!(found[0].path.starts_with("/system/fonts"));
    }

    #[test]
    fn malformed_xml_does_not_panic() {
        assert!(parse_fonts_xml("", Path::new("/")).is_empty());
        assert!(parse_fonts_xml("<font weight=\"400\">", Path::new("/")).is_empty());
        assert!(parse_fonts_xml("<fonts><fontx>a.ttf</fontx></fonts>", Path::new("/")).is_empty());
    }

    #[test]
    fn finds_the_font_that_has_the_character_and_loads_it_lazily() {
        let mut fonts = SystemFonts::new(candidates());
        assert_eq!((fonts.loaded_fonts(), fonts.mapped_files()), (0, 0));
        let cyrillic = fonts.find('Я').expect("a font for Я");
        assert_eq!(fonts.loaded_fonts(), 1, "only the covering font is loaded");
        assert!(fonts.font(cyrillic).lookup_glyph_index('Я') != 0);
        let cjk = fonts.find('日').expect("a font for 日");
        assert_ne!(cyrillic, cjk);
        // The same character is answered from the cache.
        assert_eq!(fonts.find('Я'), Some(cyrillic));
        assert_eq!(fonts.loaded_fonts(), 2);
    }

    #[test]
    fn unknown_characters_are_a_cached_miss() {
        let mut fonts = SystemFonts::new(candidates());
        assert_eq!(fonts.find('€'), None);
        assert_eq!(fonts.find('€'), None);
        assert_eq!(fonts.loaded_fonts(), 0);
    }

    #[test]
    fn missing_files_are_skipped() {
        let mut list = vec![FontCandidate {
            path: PathBuf::from("/no/such/font.ttf"),
            index: 0,
        }];
        list.extend(candidates());
        let mut fonts = SystemFonts::new(list);
        assert!(fonts.find('日').is_some());
    }

    #[test]
    fn release_drops_every_loaded_font_and_mapping() {
        let mut fonts = SystemFonts::new(candidates());
        fonts.find('Я');
        fonts.find('日');
        assert_eq!(fonts.loaded_fonts(), 2);
        assert!(fonts.mapped_files() >= 2);
        fonts.release();
        assert_eq!((fonts.loaded_fonts(), fonts.mapped_files()), (0, 0));
        // It still works afterwards.
        assert!(fonts.find('Я').is_some());
    }

    fn quads(list: &DisplayList) -> usize {
        list.cmds()
            .iter()
            .filter(|c| matches!(c, Cmd::Glyph(_)))
            .count()
    }

    #[test]
    fn engine_draws_cyrillic_and_cjk_app_names_through_the_fallback() {
        let mut engine = TextEngine::with_fallback(1.0, Box::new(SystemFonts::new(candidates())));
        let style = TextStyle::new(22.0, 700);
        let mut list = DisplayList::new();
        engine.draw(&mut list, "Яндекс 日本語", 0.0, 50.0, &style, Color::WHITE);
        // "Я", "д" and the three CJK characters come from the fallback fonts; the other
        // Cyrillic letters are not in the synthetic fonts and use the missing-glyph shape.
        // Every non-space character is drawn either way.
        assert_eq!(quads(&list), 6 + 3);
        // The synthetic boxes are 600 and 800 units tall at 22px: the CJK box is taller.
        let heights: Vec<f32> = list
            .cmds()
            .iter()
            .filter_map(|c| {
                if let Cmd::Glyph(g) = c {
                    Some(g.dest.h)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            heights[heights.len() - 1] > heights[0],
            "the CJK box is taller than the Cyrillic one"
        );
    }

    #[test]
    fn engine_release_gives_back_the_fallback_fonts() {
        let mut engine = TextEngine::with_fallback(1.0, Box::new(SystemFonts::new(candidates())));
        let mut list = DisplayList::new();
        engine.draw(
            &mut list,
            "日",
            0.0,
            50.0,
            &TextStyle::new(22.0, 500),
            Color::WHITE,
        );
        engine.release();
        assert_eq!(engine.cached_glyphs(), 0);
        // Drawing again works: the fallback reloads on demand.
        engine.draw(
            &mut list,
            "日",
            0.0,
            50.0,
            &TextStyle::new(22.0, 500),
            Color::WHITE,
        );
        assert_eq!(engine.cached_glyphs(), 1);
    }
}
