// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Enable performance.memory API in WebView2 for JS heap monitoring.
    // The frontend uses this to track memory and auto-close tabs when limit exceeded.
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--enable-precise-memory-info",
    );

    tree_pdf_viewer_lib::run()
}
