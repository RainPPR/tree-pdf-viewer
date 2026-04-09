use std::path::{Path, PathBuf};
/// 目录树节点
#[derive(Debug, Default, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub is_expanded: bool,
    pub contains_pdf: bool,
}

/// 扫描目录并构建树
pub fn build_file_tree(root_path: &Path, show_all_folders: bool) -> Vec<FileNode> {
    let mut root_node = FileNode {
        name: root_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        path: root_path.to_path_buf(),
        is_dir: true,
        children: vec![],
        is_expanded: true,
        contains_pdf: false,
    };

    if let Ok(entries) = std::fs::read_dir(root_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let child = scan_dir(&path, show_all_folders);
                if show_all_folders || child.contains_pdf {
                    root_node.children.push(child);
                }
            } else if path
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("pdf"))
            {
                root_node.children.push(FileNode {
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    path: path.clone(),
                    is_dir: false,
                    children: vec![],
                    is_expanded: false,
                    contains_pdf: true,
                });
                root_node.contains_pdf = true;
            }
        }
    }

    // 排序：目录在前，文件在后
    root_node
        .children
        .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

    vec![root_node]
}

fn scan_dir(path: &Path, show_all_folders: bool) -> FileNode {
    let mut node = FileNode {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        path: path.to_path_buf(),
        is_dir: true,
        children: vec![],
        is_expanded: false,
        contains_pdf: false,
    };

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let child = scan_dir(&p, show_all_folders);
                if show_all_folders || child.contains_pdf {
                    if child.contains_pdf {
                        node.contains_pdf = true;
                    }
                    node.children.push(child);
                }
            } else if p
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("pdf"))
            {
                node.children.push(FileNode {
                    name: p.file_name().unwrap_or_default().to_string_lossy().into(),
                    path: p.clone(),
                    is_dir: false,
                    children: vec![],
                    is_expanded: false,
                    contains_pdf: true,
                });
                node.contains_pdf = true;
            }
        }
    }

    node.children
        .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    node
}
