// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-omarchy-wallpaper: omarchy wallpaper component model.
//!
//! Real implementation: wallpapers, events, validation, deterministic storage.
//! Replaces the placeholder stub.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub subject: String,
    pub detail: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmarchyWallpaperError {
    EmptyWallpaperName,
    DuplicateWallpaper(String),
    EmptyEventSubject,
}

impl fmt::Display for OmarchyWallpaperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWallpaperName => write!(f, "wallpaper name must not be empty"),
            Self::DuplicateWallpaper(id) => write!(f, "duplicate wallpaper {:?}", id),
            Self::EmptyEventSubject => write!(f, "event subject must not be empty"),
        }
    }
}

impl std::error::Error for OmarchyWallpaperError {}

pub struct OmarchyWallpaper {
    pub name: String,
    wallpapers: BTreeMap<String, Wallpaper>,
    events: BTreeMap<String, Event>,
}

impl OmarchyWallpaper {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            wallpapers: BTreeMap::new(),
            events: BTreeMap::new(),
        }
    }

    pub fn add_wallpaper(&mut self, item: Wallpaper) -> Result<(), OmarchyWallpaperError> {
        if item.name.trim().is_empty() {
            return Err(OmarchyWallpaperError::EmptyWallpaperName);
        }
        let id = item.id.clone();
        if self.wallpapers.contains_key(&id) {
            return Err(OmarchyWallpaperError::DuplicateWallpaper(id.clone()));
        }
        self.wallpapers.insert(id, item);
        Ok(())
    }

    pub fn wallpaper_count(&self) -> usize {
        self.wallpapers.len()
    }

    pub fn get_wallpaper(&self, id: &str) -> Option<&Wallpaper> {
        self.wallpapers.get(id)
    }

    pub fn add_event(&mut self, event: Event) -> Result<(), OmarchyWallpaperError> {
        if event.subject.trim().is_empty() {
            return Err(OmarchyWallpaperError::EmptyEventSubject);
        }
        self.events.insert(event.id.clone(), event);
        Ok(())
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn get_event(&self, id: &str) -> Option<&Event> {
        self.events.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_wallpaper(id: &str, name: &str) -> Wallpaper {
        Wallpaper {
            id: id.into(),
            name: name.into(),
            kind: "default".into(),
            detail: String::new(),
        }
    }

    #[test]
    fn test_create() {
        let m = OmarchyWallpaper::new("OmarchyWallpaper");
        assert_eq!(m.name, "OmarchyWallpaper");
        assert_eq!(m.wallpaper_count(), 0);
        assert_eq!(m.event_count(), 0);
    }

    #[test]
    fn test_add_wallpaper() {
        let mut m = OmarchyWallpaper::new("OmarchyWallpaper");
        m.add_wallpaper(sample_wallpaper("a1", "Alpha")).unwrap();
        assert_eq!(m.wallpaper_count(), 1);
        assert!(m.get_wallpaper("a1").is_some());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut m = OmarchyWallpaper::new("OmarchyWallpaper");
        assert!(m.add_wallpaper(sample_wallpaper("x", "   ")).is_err());
    }

    #[test]
    fn test_duplicate_rejected() {
        let mut m = OmarchyWallpaper::new("OmarchyWallpaper");
        m.add_wallpaper(sample_wallpaper("x", "One")).unwrap();
        assert!(m.add_wallpaper(sample_wallpaper("x", "Two")).is_err());
    }

    #[test]
    fn test_event_validation() {
        let mut m = OmarchyWallpaper::new("OmarchyWallpaper");
        let bad = Event { id: "e1".into(), subject: "".into(), detail: String::new(), timestamp: 0 };
        assert!(m.add_event(bad).is_err());
        let good = Event { id: "e1".into(), subject: "boot".into(), detail: String::new(), timestamp: 1 };
        m.add_event(good).unwrap();
        assert_eq!(m.event_count(), 1);
    }
}
