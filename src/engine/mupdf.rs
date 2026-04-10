#![cfg(feature = "mupdf")]

use super::{PdfEngine, TocItem};
use anyhow::{anyhow, Result};
use egui::ColorImage;
use std::path::Path;

pub struct MuPdfEngine {
    document: Option<mupdf::Document>,
}

impl MuPdfEngine {
    pub fn new() -> Result<Self> {
        Ok(Self { document: None })
    }
}

impl PdfEngine for MuPdfEngine {
    fn open(&mut self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow!("路径无法转换为字符串"))?;
        let doc =
            mupdf::Document::open(path_str).map_err(|e| anyhow!("MuPDF 无法打开文件: {:?}", e))?;
        self.document = Some(doc);
        Ok(())
    }

    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage> {
        let doc = self
            .document
            .as_ref()
            .ok_or_else(|| anyhow!("文档未打开"))?;
        let page = doc
            .load_page(page_index as i32)
            .map_err(|e| anyhow!("MuPDF 无法加载页面: {:?}", e))?;

        let matrix = mupdf::Matrix::new_scale(zoom * 1.33, zoom * 1.33);
        let pixmap = page
            .to_pixmap(&matrix, &mupdf::Colorspace::device_rgb(), false, false)
            .map_err(|e| anyhow!("MuPDF 渲染失败: {:?}", e))?;

        let width = pixmap.width() as usize;
        let height = pixmap.height() as usize;
        let samples = pixmap.samples();

        let mut rgba = Vec::with_capacity(width * height * 4);
        for chunk in samples.chunks(3) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255);
        }

        Ok(ColorImage::from_rgba_unmultiplied([width, height], &rgba))
    }

    fn page_count(&self) -> usize {
        self.document
            .as_ref()
            .map(|d| d.page_count().unwrap_or(0) as usize)
            .unwrap_or(0)
    }

    fn get_toc(&self) -> Vec<TocItem> {
        let doc = match &self.document {
            Some(d) => d,
            None => return vec![],
        };

        let outline = match doc.outline() {
            Some(o) => o,
            None => return vec![],
        };

        let mut items = vec![];
        self.parse_outline(&outline, 0, &mut items);
        items
    }
}

impl MuPdfEngine {
    fn resolve_page_index(&self, dest: &Option<mupdf::LinkDestination>) -> usize {
        let dest = match dest {
            Some(d) => d,
            None => return 0,
        };
        match self
            .document
            .as_ref()
            .and_then(|d| d.resolve_link(dest).ok())
        {
            Some(loc) => loc.page as usize,
            None => 0,
        }
    }

    fn parse_outline(&self, outline: &mupdf::Outline, level: usize, items: &mut Vec<TocItem>) {
        let title = if outline.title.is_empty() {
            "Untitled".to_string()
        } else {
            outline.title.clone()
        };
        let page_index = self.resolve_page_index(&outline.dest);

        items.push(TocItem {
            title,
            page_index,
            children: vec![],
            level,
        });

        if !outline.down.is_empty() {
            for child in &outline.down {
                self.parse_outline(child, level + 1, items);
            }
        }
    }
}
