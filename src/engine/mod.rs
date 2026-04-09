#[cfg(feature = "mupdf")]
pub mod mupdf;
pub mod pdfium;
pub mod webview;
pub mod win_pdf;

use anyhow::Result;
use egui::ColorImage;
use std::path::Path;

/// PDF 渲染引擎的统一接口
pub trait PdfEngine: Send + Sync {
    /// 打开文档
    fn open(&mut self, path: &Path) -> Result<()>;

    /// 获取单页渲染出的位图 (RGBA)
    /// zoom: 缩放倍率，默认为 1.0 (约 72-96 DPI，取决于引擎实现)
    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage>;

    /// 获取总页数
    fn page_count(&self) -> usize;

    /// 获取文档目录 (书签树)
    fn get_toc(&self) -> Vec<TocItem>;

    /// 提取指定页面的文本信息 (可选实现)
    #[allow(dead_code)]
    fn get_text(&self, _page_index: usize) -> Result<String> {
        Ok(String::new())
    }
}

/// 目录项 (书签)
#[derive(Debug, Clone)]
pub struct TocItem {
    pub title: String,
    pub page_index: usize,
    pub children: Vec<TocItem>,
    #[allow(dead_code)]
    pub level: usize,
}

/// 支持的渲染引擎枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    #[cfg(feature = "mupdf")]
    MuPdf,
    Pdfium,
    WindowsDataPdf,
    WebView2,
}

impl Default for EngineType {
    fn default() -> Self {
        Self::Pdfium
    }
}
