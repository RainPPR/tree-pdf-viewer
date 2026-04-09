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
        let doc =
            mupdf::Document::open(path).map_err(|e| anyhow!("MuPDF 无法打开文件: {:?}", e))?;
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
        for i in 0..(width * height) {
            rgba.push(samples[i * 3]);
            rgba.push(samples[i * 3 + 1]);
            rgba.push(samples[i * 3 + 2]);
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
        let doc = if let Some(ref d) = self.document {
            d
        } else {
            return vec![];
        };

        match doc.outlines() {
            Ok(outlines) => self.parse_outline(outlines, 0),
            Err(_) => vec![],
        }
    }
}

impl MuPdfEngine {
    fn parse_outline(&self, outlines: Vec<mupdf::Outline>, level: usize) -> Vec<TocItem> {
        let mut items = vec![];

        for out in outlines {
            let title = out.title.clone().unwrap_or_else(|| "Untitled".to_string());
            let page_index = out
                .page
                .clone()
                .and_then(|loc| loc.page_number)
                .unwrap_or(-1) as usize;

            let children = if let Ok(sub_outlines) = out.outlines() {
                self.parse_outline(sub_outlines, level + 1)
            } else {
                vec![]
            };

            items.push(TocItem {
                title,
                page_index,
                children,
                level,
            });
        }

        items
    }
}

unsafe impl Send for MuPdfEngine {}
unsafe impl Sync for MuPdfEngine {}
