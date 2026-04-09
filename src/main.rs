use crate::cache::{SharedTextureCache, TextureCache};
use crate::engine::{EngineType, PdfEngine};
use crate::fs::{build_file_tree, FileNode};
use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

const APP_KEY: &str = "tree_pdf_viewer_settings";

mod cache;
mod engine;
mod fs;

#[derive(serde::Serialize, serde::Deserialize)]
struct Settings {
    engine: EngineType,
    show_all_folders: bool,
    prefetch_pages: usize,
    last_opened_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            engine: EngineType::Pdfium,
            show_all_folders: false,
            prefetch_pages: 2,
            last_opened_dir: None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
enum SidebarTab {
    Files,
    Bookmarks,
}

struct TreePdfApp {
    settings: Settings,
    file_tree: Vec<FileNode>,
    current_doc: Option<Arc<dyn PdfEngine>>,
    current_path: Option<PathBuf>,
    #[allow(dead_code)]
    search_term: String,

    // UI 状态
    sidebar_tab: SidebarTab,
    selected_node: Option<PathBuf>,
    zoom_level: f32,
    current_page: usize,

    // 渲染与缓存
    texture_cache: SharedTextureCache,
    render_rx: Receiver<RenderResult>,
    render_tx: Sender<RenderResult>,
    pending_requests: std::collections::HashSet<String>,
    jump_to_page: Option<usize>,
    show_settings: bool,
}

struct RenderResult {
    page_index: usize,
    image: egui::ColorImage,
    zoom: f32,
}

impl TreePdfApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置中文字体
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "C:/Windows/Fonts/msyh.ttc"
            ))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "microsoft_yahei".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "microsoft_yahei".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let settings: Settings = cc
            .storage
            .and_then(|s| s.get_string(APP_KEY))
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let file_tree = if let Some(ref dir) = settings.last_opened_dir {
            build_file_tree(dir, settings.show_all_folders)
        } else {
            vec![]
        };

        let (tx, rx) = channel();

        Self {
            settings,
            file_tree,
            current_doc: None,
            current_path: None,
            search_term: String::new(),
            selected_node: None,
            sidebar_tab: SidebarTab::Files,
            zoom_level: 1.0,
            current_page: 0,
            texture_cache: Arc::new(Mutex::new(TextureCache::new(50))), // 缓存 50 页
            render_rx: rx,
            render_tx: tx,
            pending_requests: std::collections::HashSet::new(),
            jump_to_page: None,
            show_settings: false,
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        let engine_type = self.settings.engine;
        let engine_result: Result<Arc<dyn PdfEngine>> = match engine_type {
            #[cfg(feature = "mupdf")]
            EngineType::MuPdf => {
                crate::engine::mupdf::MuPdfEngine::new().map(|e| Arc::new(e) as Arc<dyn PdfEngine>)
            }
            EngineType::Pdfium => crate::engine::pdfium::PdfiumEngine::new()
                .map(|e| Arc::new(e) as Arc<dyn PdfEngine>),
            EngineType::WindowsDataPdf => crate::engine::win_pdf::WinPdfEngine::new()
                .map(|e| Arc::new(e) as Arc<dyn PdfEngine>),
            EngineType::WebView2 => crate::engine::webview::WebView2Engine::new()
                .map(|e| Arc::new(e) as Arc<dyn PdfEngine>),
        };

        if let Ok(mut engine) = engine_result {
            // 注意：因为我们要调用 open(&mut self)，我们需要解包 Arc 或者直接在闭包里操作。
            // 之前的实现可能有误，这里应当确保我们拿到的是独占所有权或在同步环境下处理。
            // 由于 PdfEngine Trait 定义为 open(&mut self)，我们这里暂时直接获取 mut。
            if let Some(engine_mut) = Arc::get_mut(&mut engine) {
                if engine_mut.open(&path).is_ok() {
                    self.current_doc = Some(engine);
                    self.current_path = Some(path);
                    self.current_page = 0;
                    self.texture_cache.lock().unwrap().clear();
                    self.pending_requests.clear();
                }
            } else {
                // 如果 Arc 无法获取 mut (说明有其他地方持有)，则尝试重新创建一个
                // 简化起见，这里直接使用刚刚初始化的 engine (假设只有一个持有者)
                // 实际重写中通常在 open 前不进行 Arc 包装更为稳妥
            }
        }
    }

    fn request_render(&mut self, page_index: usize) {
        let zoom = self.zoom_level;
        let request_id = format!("{}_{}", page_index, zoom);

        if self.pending_requests.contains(&request_id) {
            return;
        }

        if let Some(ref doc) = self.current_doc {
            self.pending_requests.insert(request_id.clone());
            let tx = self.render_tx.clone();
            let doc_clone = Arc::clone(doc);

            // 使用标准线程执行渲染，避免阻塞 UI
            std::thread::spawn(move || {
                if let Ok(image) = doc_clone.render_page(page_index, zoom) {
                    let _ = tx.send(RenderResult {
                        page_index,
                        image,
                        zoom,
                    });
                }
            });
        }
    }

    fn open_folder(&mut self, path: PathBuf) {
        self.settings.last_opened_dir = Some(path.clone());
        self.file_tree = build_file_tree(&path, self.settings.show_all_folders);
    }
}

impl eframe::App for TreePdfApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(&self.settings) {
            storage.set_string(APP_KEY, json);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();

        // 处理渲染结果
        while let Ok(result) = self.render_rx.try_recv() {
            if result.zoom == self.zoom_level {
                let texture_name = format!("page_{}_{}", result.page_index, result.zoom);
                let texture =
                    ctx.load_texture(&texture_name, result.image, egui::TextureOptions::LINEAR);
                self.texture_cache
                    .lock()
                    .unwrap()
                    .insert(texture_name, texture);
            }
        }

        // 顶部工具栏
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📁 打开文件夹").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.open_folder(path);
                    }
                }

                ui.separator();

                ui.label("引擎:");
                let old_engine = self.settings.engine;
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.settings.engine))
                    .show_ui(ui, |ui| {
                        #[cfg(feature = "mupdf")]
                        ui.selectable_value(&mut self.settings.engine, EngineType::MuPdf, "MuPDF");
                        ui.selectable_value(
                            &mut self.settings.engine,
                            EngineType::Pdfium,
                            "PDFium",
                        );
                        ui.selectable_value(
                            &mut self.settings.engine,
                            EngineType::WindowsDataPdf,
                            "Windows.Data.Pdf",
                        );
                        ui.selectable_value(
                            &mut self.settings.engine,
                            EngineType::WebView2,
                            "WebView2",
                        );
                    });

                if self.settings.engine != old_engine {
                    if let Some(path) = self.current_path.clone() {
                        self.open_file(path);
                    }
                }

                ui.separator();

                ui.label("缩放:");
                if ui
                    .add(egui::Slider::new(&mut self.zoom_level, 0.5..=5.0).logarithmic(true))
                    .changed()
                {
                    // 缩放改变时清除缓存并重新请求
                    self.texture_cache.lock().unwrap().clear();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ 设置").clicked() {
                        self.show_settings = true;
                    }
                });
            });
        });

        // 底部状态栏
        egui::Panel::bottom("status_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(ref path) = self.current_path {
                    ui.label(format!(
                        "文件: {}",
                        path.file_name().unwrap().to_string_lossy()
                    ));
                } else {
                    ui.label("未打开文件");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("内存限制: 1.0 GB")); // 示意
                });
            });
        });

        // 左侧目录树
        egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(250.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Files, "📂 文件");
                    ui.selectable_value(&mut self.sidebar_tab, SidebarTab::Bookmarks, "📑 书签");
                });
                ui.separator();

                match self.sidebar_tab {
                    SidebarTab::Files => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut nodes_to_render = self.file_tree.clone();
                            for node in &mut nodes_to_render {
                                if let Some(path) = self.render_tree_node(ui, node) {
                                    self.open_file(path);
                                }
                            }
                        });
                    }
                    SidebarTab::Bookmarks => {
                        if let Some(ref doc) = self.current_doc {
                            let toc = doc.get_toc();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for item in toc {
                                    self.render_toc_item(ui, &item);
                                }
                            });
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.label("未打开文档");
                            });
                        }
                    }
                }
            });

        // 右侧预览区
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(ref doc) = self.current_doc {
                let total_pages = doc.page_count();
                let zoom = self.zoom_level;
                let row_height = 810.0 * zoom;

                let scroll_area = egui::ScrollArea::vertical()
                    .id_salt("pdf_scroll")
                    .auto_shrink([false, false]);

                let texture_cache_clone = Arc::clone(&self.texture_cache);
                let requested_pages = Arc::new(Mutex::new(Vec::new()));
                let requested_pages_inner = Arc::clone(&requested_pages);

                scroll_area.show_rows(ui, row_height, total_pages, |ui, range| {
                    for i in range {
                        let texture_name = format!("page_{}_{}", i, zoom);
                        let texture = texture_cache_clone.lock().unwrap().get(&texture_name);

                        if let Some(tex) = texture {
                            ui.image((tex.id(), tex.size_vec2()));
                            ui.add_space(10.0);
                        } else {
                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(600.0 * zoom, 800.0 * zoom),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(rect, 0.0, egui::Color32::from_gray(30));
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "加载中...",
                                egui::FontId::proportional(20.0 * zoom),
                                egui::Color32::GRAY,
                            );
                            ui.add_space(10.0);

                            requested_pages_inner.lock().unwrap().push(i);
                        }
                    }
                });

                if let Some(target_page) = self.jump_to_page.take() {
                    let scroll_offset = target_page as f32 * row_height;
                    ui.scroll_with_delta(egui::vec2(0.0, -scroll_offset));
                }

                let pages = requested_pages.lock().unwrap().clone();
                for page_idx in pages {
                    self.request_render(page_idx);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("选择一个 PDF 文件以查看预览");
                });
            }
        });

        // 设置对话框
        if self.show_settings {
            egui::Window::new("⚙ 设置")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.heading("设置");
                    ui.separator();

                    ui.label("PDF 渲染引擎:");
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("engine_selector")
                            .selected_text(format!("{:?}", self.settings.engine))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.settings.engine,
                                    EngineType::Pdfium,
                                    "PDFium",
                                );
                                ui.selectable_value(
                                    &mut self.settings.engine,
                                    EngineType::WindowsDataPdf,
                                    "Windows Data PDF",
                                );
                                ui.selectable_value(
                                    &mut self.settings.engine,
                                    EngineType::WebView2,
                                    "WebView2",
                                );
                            });
                    });

                    ui.separator();
                    ui.label("预取页数:");
                    ui.add(
                        egui::Slider::new(&mut self.settings.prefetch_pages, 0..=10)
                            .text("prefetch"),
                    );

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            self.show_settings = false;
                        }
                        if ui.button("取消").clicked() {
                            self.show_settings = false;
                        }
                    });
                });
        }
    }
}

impl TreePdfApp {
    fn render_tree_node(&mut self, ui: &mut egui::Ui, node: &mut FileNode) -> Option<PathBuf> {
        let mut clicked_path = None;
        if node.is_dir {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                egui::Id::new(&node.path),
                node.is_expanded,
            )
            .show_header(ui, |ui| {
                ui.label(format!("📁 {}", node.name));
            })
            .body(|ui| {
                for child in &mut node.children {
                    if let Some(p) = self.render_tree_node(ui, child) {
                        clicked_path = Some(p);
                    }
                }
            });
        } else {
            let is_selected = self.selected_node.as_ref() == Some(&node.path);
            if ui
                .selectable_label(is_selected, format!("📄 {}", node.name))
                .clicked()
            {
                self.selected_node = Some(node.path.clone());
                clicked_path = Some(node.path.clone());
            }
        }
        clicked_path
    }

    fn render_toc_item(&mut self, ui: &mut egui::Ui, item: &crate::engine::TocItem) {
        let label = format!("{} (P{})", item.title, item.page_index + 1);
        let collapser_id = ui.make_persistent_id(&item.title);

        if !item.children.is_empty() {
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                collapser_id,
                false,
            )
            .show_header(ui, |ui| {
                if ui.button(&label).clicked() {
                    self.jump_to_page = Some(item.page_index);
                }
            })
            .body(|ui| {
                for child in &item.children {
                    self.render_toc_item(ui, child);
                }
            });
        } else {
            if ui
                .selectable_label(self.current_page == item.page_index, &label)
                .clicked()
            {
                self.jump_to_page = Some(item.page_index);
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Tree PDF Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "tree-pdf-viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(TreePdfApp::new(cc)))),
    )
}
