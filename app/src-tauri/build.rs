fn main() {
    // Declaring the app commands makes them subject to capabilities: a command
    // is unreachable from the webview unless capabilities/ grants it.
    let manifest = tauri_build::AppManifest::new().commands(&["get_app_info"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("failed to run the Tauri build script");

    // tauri-build embeds the Common Controls v6 manifest in bins only. Without it, test
    // executables that link Tauri exit with STATUS_ENTRYPOINT_NOT_FOUND before any test runs.
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.ends_with("windows-msvc") {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' \
             name='Microsoft.Windows.Common-Controls' version='6.0.0.0' \
             processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
