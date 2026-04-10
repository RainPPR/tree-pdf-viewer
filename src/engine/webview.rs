use super::{PdfEngine, TocItem};
use anyhow::Result;
use egui::ColorImage;
use std::path::Path;

pub struct WebView2Engine {
    // WebView2 通常需要关联一个窗口句柄 (HWND)
    // 在 egui 中通过自定义渲染节点或者原生窗口集成比较复杂
    // 这里先实现一个基础接口
    current_path: Option<String>,
}

impl WebView2Engine {
    pub fn new() -> Result<Self> {
        Ok(Self { current_path: None })
    }
}

impl PdfEngine for WebView2Engine {
    fn open(&mut self, path: &Path) -> Result<()> {
        self.current_path = Some(path.to_string_lossy().into());
        Ok(())
    }

    fn render_page(&self, _page_index: usize, _zoom: f32) -> Result<ColorImage> {
        let width = 600;
        let height = 800;
        let mut pixels = vec![0u8; width * height * 4];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                pixels[idx] = 45;
                pixels[idx + 1] = 45;
                pixels[idx + 2] = 50;
                pixels[idx + 3] = 255;
            }
        }

        Ok(ColorImage::from_rgba_unmultiplied([width, height], &pixels))
    }

    fn page_count(&self) -> usize {
        1
    }

    fn get_toc(&self) -> Vec<TocItem> {
        vec![]
    }
}

unsafe impl Send for WebView2Engine {}
unsafe impl Sync for WebView2Engine {}
