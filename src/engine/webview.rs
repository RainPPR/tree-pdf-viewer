use super::{PdfEngine, TocItem};
use anyhow::{anyhow, Result};
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
        // WebView2 的初始化和文档加载通常是异步的
        Ok(())
    }

    fn render_page(&self, _page_index: usize, _zoom: f32) -> Result<ColorImage> {
        // 对于 WebView2，我们通常不将其渲染结果导回到 CPU 位图
        // 而是直接在 UI 上覆盖一个浏览器窗口。
        // 为了兼容 Trait 接口，返回一个特定的提示占位图。
        Err(anyhow!(
            "WebView2 引擎目前通过原生窗口覆盖层直接显示，不支持 render_page 接口。"
        ))
    }

    fn page_count(&self) -> usize {
        0 // WebView2 中难以同步获取
    }

    fn get_toc(&self) -> Vec<TocItem> {
        vec![]
    }
}

unsafe impl Send for WebView2Engine {}
unsafe impl Sync for WebView2Engine {}
