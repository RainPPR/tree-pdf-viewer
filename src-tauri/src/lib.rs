use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub children: Option<Vec<TreeNode>>,
}

/// Recursively scan a directory and build a tree containing only PDF files
/// and folders that contain at least one PDF (directly or recursively).
fn scan_for_pdfs(dir: &Path) -> Option<TreeNode> {
    let mut children = Vec::new();
    let mut has_pdf = false;

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    // Collect entries first, then sort for consistent ordering
    let mut sorted_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted_entries.sort_by_key(|e| e.file_name());

    for entry in sorted_entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            // Skip hidden directories
            if name.starts_with('.') {
                continue;
            }
            if let Some(subtree) = scan_for_pdfs(&path) {
                children.push(subtree);
                has_pdf = true;
            }
        } else if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("pdf")) {
            has_pdf = true;
            children.push(TreeNode {
                name,
                path: path.to_string_lossy().to_string(),
                is_folder: false,
                children: None,
            });
        }
    }

    if has_pdf {
        Some(TreeNode {
            name: dir.file_name()?.to_string_lossy().to_string(),
            path: dir.to_string_lossy().to_string(),
            is_folder: true,
            children: if children.is_empty() {
                None
            } else {
                Some(children)
            },
        })
    } else {
        None
    }
}

#[tauri::command]
fn scan_pdf_tree(root: String) -> Result<Option<TreeNode>, String> {
    let path = Path::new(&root);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", root));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", root));
    }
    Ok(scan_for_pdfs(path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![scan_pdf_tree])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
