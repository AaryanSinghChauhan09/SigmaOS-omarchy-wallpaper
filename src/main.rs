//! SigmaOS-wallpaper — wallpaper CLI.
//!
//! Usage:
//!   omarchy-wallpaper list
//!   omarchy-wallpaper outputs
//!   omarchy-wallpaper set <id> [output]
//!   omarchy-wallpaper current [output]
//!   omarchy-wallpaper cycle [output]
//!   omarchy-wallpaper random [output]
//!   omarchy-wallpaper theme [output]   (suggested theme)

use sigmaos_wallpaper::{OmarchyWallpaper, Rgb, Wallpaper};
use std::process::exit;

fn seed() -> OmarchyWallpaper {
    let mut m = OmarchyWallpaper::new();
    m.add_output("eDP-1");
    m.add_output("HDMI-A-1");
    let mut defs = vec![
        ("catppuccin", "Catppuccin Mocha", "/usr/share/backgrounds/catppuccin.png", vec![(30,30,46)], Some("catppuccin-mocha")),
        ("tokyo-night", "Tokyo Night", "/usr/share/backgrounds/tokyo-night.png", vec![(26,27,38)], Some("tokyo-night")),
        ("nord", "Nord", "/usr/share/backgrounds/nord.png", vec![(46,52,64)], None),
        ("gruvbox", "Gruvbox Dark", "/usr/share/backgrounds/gruvbox.png", vec![(40,40,40)], Some("gruvbox-dark")),
        ("latte", "Catppuccin Latte", "/usr/share/backgrounds/latte.png", vec![(239,241,245)], Some("catppuccin-latte")),
    ];
    for (id, name, path, dom, hint) in defs.drain(..) {
        let mut w = Wallpaper::new(id, name, path);
        w.width = 1920; w.height = 1080;
        w.dominant = dom.iter().map(|&(r,g,b)| Rgb{r,g,b}).collect();
        w.theme_hint = hint.map(|s| s.to_string());
        let _ = m.register(w);
    }
    m
}

fn usage() -> ! {
    eprintln!("usage: omarchy-wallpaper <list|outputs|set|current|cycle|random|theme> [args]");
    exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { usage(); }
    let state = OmarchyWallpaper::state_path();
    let mut m = OmarchyWallpaper::load_from(&state).unwrap_or_else(|_| seed());
    let cmd = args[0].as_str();
    let rest = &args[1..];
    let out = rest.get(0).map(|s| s.as_str()).unwrap_or("eDP-1");

    let result: Result<(), String> = (|| -> Result<(), String> { match cmd {
        "list" => {
            for w in m.wallpapers() {
                println!("{:<12} {:<20} {:<6} {}", w.id, w.name, w.suggested_variant(), w.path);
            }
            Ok(())
        }
        "outputs" => {
            for o in ["eDP-1", "HDMI-A-1"] { println!("{}", m.status_line(o)); }
            Ok(())
        }
        "set" => {
            let id = rest.first().map(|s| s.as_str()).unwrap_or_else(|| usage());
            let output = rest.get(1).map(|s| s.as_str());
            m.set(output, id).map(|_| println!("set {} -> {}", output.unwrap_or("all"), id))
                .map_err(|e| e.to_string())
        }
        "current" => { println!("{}", m.status_line(out)); Ok(()) }
        "cycle" => m.cycle(out).map(|id| println!("cycled {} -> {}", out, id)).map_err(|e| e.to_string()),
        "random" => m.random(out).map(|id| println!("random {} -> {}", out, id)).map_err(|e| e.to_string()),
        "theme" => {
            match m.theme_hint(out) {
                Some(t) => { println!("{t}"); Ok(()) }
                None => Err(format!("no wallpaper on {out}")),
            }
        }
        _ => usage(),
    } })();

    match result {
        Ok(()) => {
            if let Err(e) = m.save_to(&state) { eprintln!("omarchy-wallpaper: warn: save: {e}"); }
        }
        Err(e) => { eprintln!("omarchy-wallpaper: {e}"); exit(1); }
    }
}
