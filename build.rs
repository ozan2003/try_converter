//! Detects whether the optional window icon is available to embed.
//! The absence of the icon shouldn't prevent the app from building but its nice
//! to have nontheless.

use std::path::PathBuf;

fn main()
{
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rustc-check-cfg=cfg(has_icon)");

    let has_icon =
        std::env::var_os("CARGO_MANIFEST_DIR").is_some_and(|manifest_dir| {
            PathBuf::from(manifest_dir)
                .join("assets")
                .join("icon.png")
                .is_file()
        });

    if has_icon
    {
        println!("cargo:rustc-cfg=has_icon");
    }
}
