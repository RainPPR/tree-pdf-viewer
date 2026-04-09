use super::{PdfEngine, TocItem};
use anyhow::{anyhow, Result};
use egui::ColorImage;
use pdfium_render::prelude::*;
use std::path::Path;
use std::sync::Arc;

pub struct PdfiumEngine {
    pdfium: Arc<Pdfium>,
    document: Option<PdfDocument<'static>>,
}

impl PdfiumEngine {
    pub fn new() -> Result<Self> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name())
                .or_else(|_| Pdfium::bind_to_library("pdfium.dll"))
                .map_err(|e| anyhow!("无法加载 pdfium.dll: {}. 请确保它在程序目录下。", e))?,
        );

        Ok(Self {
            pdfium: Arc::new(pdfium),
            document: None,
        })
    }
}

impl PdfEngine for PdfiumEngine {
    fn open(&mut self, path: &Path) -> Result<()> {
        let doc = self.pdfium.load_pdf_from_file(path, None)?;
        self.document =
            Some(unsafe { std::mem::transmute::<PdfDocument<'_>, PdfDocument<'static>>(doc) });
        Ok(())
    }

    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage> {
        let doc = self
            .document
            .as_ref()
            .ok_or_else(|| anyhow!("文档未打开"))?;

        let page = doc.pages().get(page_index as i32)?;

        let width_pt = page.width().value;
        let height_pt = page.height().value;

        let render_config = PdfRenderConfig::new()
            .set_target_width((width_pt * 1.33 * zoom) as i32)
            .set_target_height((height_pt * 1.33 * zoom) as i32);

        let bitmap = page.render_with_config(&render_config)?;
        let width = bitmap.width() as usize;
        let height = bitmap.height() as usize;

        let pixels = bitmap.as_rgba_bytes();
        Ok(ColorImage::from_rgba_unmultiplied([width, height], &pixels))
    }

    fn page_count(&self) -> usize {
        self.document
            .as_ref()
            .map(|d| d.pages().len() as usize)
            .unwrap_or(0)
    }

    fn get_toc(&self) -> Vec<TocItem> {
        let doc = match &self.document {
            Some(d) => d,
            None => return vec![],
        };
        self.parse_bookmarks(doc.bookmarks().iter(), 0)
    }
}

impl PdfiumEngine {
    fn parse_bookmarks<'a>(
        &self,
        mut iter: PdfBookmarksIterator<'a>,
        level: usize,
    ) -> Vec<TocItem> {
        let mut items = vec![];

        while let Some(bm) = iter.next() {
            let children = self.parse_bookmarks(bm.iter_direct_children(), level + 1);

            let page_index = bm
                .destination()
                .and_then(|d| d.page_index().ok())
                .map(|idx| idx as usize)
                .unwrap_or(0);

            items.push(TocItem {
                title: bm.title().unwrap_or_else(|| "无标题".to_string()),
                page_index,
                children,
                level,
            });
        }

        items
    }
}

unsafe impl Send for PdfiumEngine {}
unsafe impl Sync for PdfiumEngine {}
