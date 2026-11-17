//! Integration tests for SigmaOS-omarchy-wallpaper.

use sigmaos_omarchy_wallpaper_lib::Component;

#[test]
fn component_initialises_and_runs() {
    let mut c = Component::new();
    c.init().expect("init should succeed");
    c.run().expect("run should succeed after init");
}
