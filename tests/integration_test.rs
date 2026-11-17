// Integration test for SigmaOS-omarchy-wallpaper
// Verifies the crate builds and links as a dependency.
#![allow(unused_imports)]

use omarchy_wallpaper as _;

#[test]
fn crate_links_and_runs() {
    assert_eq!(2 + 2, 4);
}
