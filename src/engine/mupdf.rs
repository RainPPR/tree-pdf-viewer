#![cfg(feature = "mupdf")]

use super::{PdfEngine, TocItem};
use anyhow::{anyhow, Result};
use egui::ColorImage;
use std::path::Path;

pub struct MuPdfEngine {
    context: mupdf::Context,
    document: Option<mupdf::Document>,
}

impl MuPdfEngine {
    pub fn new() -> Result<Self> {
        let context = mupdf::Context::new()
            .map_err(|e| anyhow!("无法初始化 MuPDF Context: {:?}", e))?;
        Ok(Self {
            context,
            document: None,
        })
    }
}

impl PdfEngine for MuPdfEngine {
    fn open(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();
        let doc = mupdf::Document::open(&self.context, &path_str)
            .map_err(|e| anyhow!("MuPDF 无法打开文件: {:?}", e))?;
        self.document = Some(doc);
        Ok(())
    }

    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage> {
        let doc = self.document.as_ref().ok_or_else(|| anyhow!("文档未打开"))?;
        let page = doc.load_page(page_index as i32)
            .map_err(|e| anyhow!("MuPDF 无法加载页面: {:?}", e))?;
        
        // 计算缩放矩阵
        let matrix = mupdf::Matrix::new_scale(zoom * 1.33, zoom * 1.33);
        let pixmap = page.to_pixmap(&matrix, &mupdf::Colorspace::device_rgb(), 0.0)
            .map_err(|e| anyhow!("MuPDF 渲染失败: {:?}", e))?;
            
        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;
        let samples = pixmap.samples(); // RGB 数据
        
        // 转换为 egui 的 ColorImage (RGBA)
        let mut rgba = Vec::with_capacity(width * height * 4);
        for i in 0..(width * height) {
            rgba.push(samples[i * 3]);     // R
            rgba.push(samples[i * 3 + 1]); // G
            rgba.push(samples[i * 3 + 2]); // B
            rgba.push(255);                 // A
        }
        
        Ok(ColorImage::from_rgba_unmultiplied([width, height], &rgba))
    }

    fn page_count(&self) -> usize {
        self.document.as_ref().map(|d| d.count_pages().unwrap_or(0) as usize).unwrap_or(0)
    }

    fn get_toc(&self) -> Vec<TocItem> {
        let doc = if let Some(ref d) = self.document { d } else { return vec![]; };
        let outline = match doc.load_outline() {
            Ok(o) => o,
            Err(_) => return vec![],
        };
        self.parse_outline(Some(outline), 0)
    }
}

impl MuPdfEngine {
    fn parse_outline(&self, outline: Option<mupdf::Outline>, level: usize) -> Vec<TocItem> {
        let mut items = vec![];
        let mut current = outline;
        
        while let Some(out) = current {
            let mut children = vec![];
            if let Some(first_child) = out.first_child() {
                children = self.parse_outline(Some(first_child), level + 1);
            }
            
            items.push(TocItem {
                title: out.title().unwrap_or_else(|| "Untitled".to_string()),
                page_index: out.page() as usize,
                children,
                level,
            });
            
            current = out.next();
        }
        
        items
    }
}

unsafe impl Send for MuPdfEngine {}
unsafe impl Sync for MuPdfEngine {}
