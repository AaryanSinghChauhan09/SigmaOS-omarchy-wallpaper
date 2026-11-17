// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-omarchy-wallpaper — wallpaper manager with theme coupling.
//!
//! Registry of wallpapers (image path + extracted dominant colors), set/
//! cycle/random per output, and a hook that suggests the matching theme
//! (Wallust-style: wallpaper drives the palette).

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }
impl Rgb {
    pub fn hex(self) -> String { format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b) }
    pub fn luminance(self) -> f32 {
        let f = |c: u8| { let c = c as f32 / 255.0; if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) } };
        0.2126 * f(self.r) + 0.7152 * f(self.g) + 0.0722 * f(self.b)
    }
    pub fn is_light(self) -> bool { self.luminance() > 0.5 }
}

#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub id: String,
    pub name: String,
    pub path: String,               // image file path or resource URI
    pub dominant: Vec<Rgb>,         // extracted palette (avg/muted/etc)
    pub width: u32,
    pub height: u32,
    pub theme_hint: Option<String>, // suggested omarchy-theme name
}

impl Wallpaper {
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: id.into(), name: name.into(), path: path.into(),
            dominant: Vec::new(), width: 0, height: 0, theme_hint: None,
        }
    }
    /// Guess dark/light variant from dominant colors.
    pub fn suggested_variant(&self) -> &'static str {
        if let Some(c) = self.dominant.first() {
            if c.is_light() { "light" } else { "dark" }
        } else { "dark" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WallpaperError {
    EmptyId,
    Duplicate(String),
    NoSuchWallpaper(String),
    NoWallpaper,
    EmptyPath,
}
impl fmt::Display for WallpaperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(f, "wallpaper id must not be empty"),
            Self::Duplicate(id) => write!(f, "duplicate wallpaper {id}"),
            Self::NoSuchWallpaper(id) => write!(f, "no such wallpaper {id}"),
            Self::NoWallpaper => write!(f, "no wallpaper set"),
            Self::EmptyPath => write!(f, "wallpaper path must not be empty"),
        }
    }
}
impl std::error::Error for WallpaperError {}

#[derive(Debug, Clone)]
pub struct Output {
    pub name: String,               // "eDP-1", "HDMI-A-1"
    pub wallpaper: Option<String>,  // wallpaper id
}

pub struct OmarchyWallpaper {
    wallpapers: BTreeMap<String, Wallpaper>,
    outputs: BTreeMap<String, Output>,
    random_seed: u64,
}

impl OmarchyWallpaper {
    pub fn new() -> Self {
        Self { wallpapers: BTreeMap::new(), outputs: BTreeMap::new(), random_seed: 0x9e3779b9 }
    }

    pub fn register(&mut self, w: Wallpaper) -> Result<(), WallpaperError> {
        if w.id.trim().is_empty() { return Err(WallpaperError::EmptyId); }
        if w.path.trim().is_empty() { return Err(WallpaperError::EmptyPath); }
        if self.wallpapers.contains_key(&w.id) { return Err(WallpaperError::Duplicate(w.id)); }
        self.wallpapers.insert(w.id.clone(), w);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Wallpaper> { self.wallpapers.get(id) }
    pub fn wallpapers(&self) -> impl Iterator<Item = &Wallpaper> { self.wallpapers.values() }
    pub fn count(&self) -> usize { self.wallpapers.len() }

    pub fn add_output(&mut self, name: impl Into<String>) {
        let n = name.into();
        self.outputs.entry(n.clone()).or_insert(Output { name: n, wallpaper: None });
    }

    /// Set wallpaper on an output (or all outputs if name is None).
    pub fn set(&mut self, output: Option<&str>, id: &str) -> Result<(), WallpaperError> {
        if !self.wallpapers.contains_key(id) { return Err(WallpaperError::NoSuchWallpaper(id.into())); }
        match output {
            Some(o) => {
                let out = self.outputs.get_mut(o).ok_or_else(|| WallpaperError::NoSuchWallpaper(o.into()))?;
                out.wallpaper = Some(id.into());
            }
            None => {
                if self.outputs.is_empty() { self.add_output("eDP-1"); }
                for out in self.outputs.values_mut() { out.wallpaper = Some(id.into()); }
            }
        }
        Ok(())
    }

    pub fn current(&self, output: &str) -> Option<&Wallpaper> {
        self.outputs.get(output)?.wallpaper.as_ref().and_then(|id| self.wallpapers.get(id))
    }

    /// Deterministic pseudo-random pick (xorshift) for `cycle`/`random`.
    fn next_rand(&mut self) -> u64 {
        let mut x = self.random_seed;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.random_seed = x; x
    }

    /// Cycle to next wallpaper (sorted id order) on an output.
    pub fn cycle(&mut self, output: &str) -> Result<String, WallpaperError> {
        let ids: Vec<String> = self.wallpapers.keys().cloned().collect();
        if ids.is_empty() { return Err(WallpaperError::NoWallpaper); }
        let cur = self.outputs.get(output).and_then(|o| o.wallpaper.clone());
        let next = match cur {
            None => ids[0].clone(),
            Some(c) => {
                let i = ids.iter().position(|x| *x == c).unwrap_or(0);
                ids[(i + 1) % ids.len()].clone()
            }
        };
        self.set(Some(output), &next)?;
        Ok(next)
    }

    /// Random wallpaper on an output.
    pub fn random(&mut self, output: &str) -> Result<String, WallpaperError> {
        let ids: Vec<String> = self.wallpapers.keys().cloned().collect();
        if ids.is_empty() { return Err(WallpaperError::NoWallpaper); }
        let idx = (self.next_rand() as usize) % ids.len();
        let pick = ids[idx].clone();
        self.set(Some(output), &pick)?;
        Ok(pick)
    }

    /// Suggest the theme variant for the current wallpaper of an output.
    pub fn theme_hint(&self, output: &str) -> Option<String> {
        let w = self.current(output)?;
        w.theme_hint.clone().or_else(|| Some(format!("auto-{}", w.suggested_variant())))
    }

    pub fn status_line(&self, output: &str) -> String {
        match self.current(output) {
            Some(w) => format!("{}: {} ({})", output, w.name, w.suggested_variant()),
            None => format!("{}: <none>", output),
        }
    }
}

impl Default for OmarchyWallpaper {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> OmarchyWallpaper {
        let mut m = OmarchyWallpaper::new();
        m.add_output("eDP-1"); m.add_output("HDMI-A-1");
        let mut w1 = Wallpaper::new("catppuccin", "Catppuccin", "/usr/share/wallpapers/catppuccin.png");
        w1.dominant = vec![Rgb { r: 30, g: 30, b: 46 }];
        w1.width = 1920; w1.height = 1080; w1.theme_hint = Some("catppuccin-mocha".into());
        let mut w2 = Wallpaper::new("nord", "Nord", "/usr/share/wallpapers/nord.png");
        w2.dominant = vec![Rgb { r: 46, g: 52, b: 64 }];
        let mut w3 = Wallpaper::new("latte", "Latte", "/usr/share/wallpapers/latte.png");
        w3.dominant = vec![Rgb { r: 239, g: 241, b: 245 }];
        w3.theme_hint = Some("catppuccin-latte".into());
        for w in [w1, w2, w3] { m.register(w).unwrap(); }
        m
    }

    #[test]
    fn register_and_get() {
        let mut m = mgr();
        assert_eq!(m.count(), 3);
        assert!(m.get("nord").is_some());
        assert!(m.register(Wallpaper::new(" ", "x", "/p")).is_err());
        assert!(m.register(Wallpaper::new("y", "x", "  ")).is_err());
        assert!(m.register(Wallpaper::new("nord", "dup", "/p")).is_err());
    }

    #[test]
    fn set_and_current() {
        let mut m = mgr();
        m.set(Some("eDP-1"), "nord").unwrap();
        assert_eq!(m.current("eDP-1").unwrap().name, "Nord");
        assert!(m.current("HDMI-A-1").is_none());
        m.set(None, "catppuccin").unwrap(); // all outputs
        assert!(m.current("HDMI-A-1").is_some());
    }

    #[test]
    fn cycle_and_random() {
        let mut m = mgr();
        let first = m.cycle("eDP-1").unwrap();
        let second = m.cycle("eDP-1").unwrap();
        let third = m.cycle("eDP-1").unwrap();
        assert_ne!(first, second);
        assert_ne!(second, third);
        // wraps
        let fourth = m.cycle("eDP-1").unwrap();
        assert_eq!(fourth, first);
        let r = m.random("eDP-1").unwrap();
        assert!(m.get(&r).is_some());
    }

    #[test]
    fn theme_hint_from_wallpaper() {
        let mut m = mgr();
        m.set(Some("eDP-1"), "catppuccin").unwrap();
        assert_eq!(m.theme_hint("eDP-1").unwrap(), "catppuccin-mocha");
        m.set(Some("eDP-1"), "nord").unwrap();
        assert_eq!(m.theme_hint("eDP-1").unwrap(), "auto-dark"); // no hint -> variant
        m.set(Some("eDP-1"), "latte").unwrap();
        assert_eq!(m.theme_hint("eDP-1").unwrap(), "catppuccin-latte");
    }

    #[test]
    fn errors_and_status() {
        let mut m = OmarchyWallpaper::new();
        assert!(m.set(Some("eDP-1"), "x").is_err()); // no such wallpaper
        assert!(m.cycle("eDP-1").is_err());           // empty registry
        let mut m2 = mgr();
        assert!(m2.set(Some("eDP-1"), "nope").is_err());
        m2.set(Some("eDP-1"), "nord").unwrap();
        assert!(m2.status_line("eDP-1").contains("Nord"));
    }
}
