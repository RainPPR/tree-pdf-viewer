use super::{PdfEngine, TocItem};
use anyhow::{anyhow, Result};
use egui::ColorImage;
use std::path::Path;
use windows::Data::Pdf::{PdfDocument, PdfPageRenderOptions};
use windows::Storage::StorageFile;
use windows::Storage::Streams::InMemoryRandomAccessStream;
// HWND 已移除，因为目前未使用原生窗口句柄操作

pub struct WinPdfEngine {
    document: Option<PdfDocument>,
}

impl WinPdfEngine {
    pub fn new() -> Result<Self> {
        Ok(Self { document: None })
    }
}

impl PdfEngine for WinPdfEngine {
    fn open(&mut self, path: &Path) -> Result<()> {
        let abs_path = std::fs::canonicalize(path)?;
        let path_str = abs_path.to_string_lossy().to_string();

        let hpath = windows::core::HSTRING::from(&path_str);

        let file = StorageFile::GetFileFromPathAsync(&hpath)
            .map_err(|e| anyhow!("获取文件失败: {:?}", e))?
            .get()
            .map_err(|e| anyhow!("等待文件加载失败: {:?}", e))?;

        let doc = PdfDocument::LoadFromFileAsync(&file)
            .map_err(|e| anyhow!("加载PDF失败: {:?}", e))?
            .get()
            .map_err(|e| anyhow!("等待PDF加载失败: {:?}", e))?;

        self.document = Some(doc);
        Ok(())
    }

    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage> {
        let doc = self
            .document
            .as_ref()
            .ok_or_else(|| anyhow!("文档未打开"))?;
        let page = doc
            .GetPage(page_index as u32)
            .map_err(|e| anyhow!("获取页面失败: {:?}", e))?;

        let stream =
            InMemoryRandomAccessStream::new().map_err(|e| anyhow!("创建流失败: {:?}", e))?;
        let options =
            PdfPageRenderOptions::new().map_err(|e| anyhow!("创建渲染选项失败: {:?}", e))?;

        let size = page
            .Size()
            .map_err(|e| anyhow!("获取页面尺寸失败: {:?}", e))?;

        options
            .SetDestinationWidth((size.Width * zoom) as u32)
            .map_err(|e| anyhow!("设置宽度失败: {:?}", e))?;
        options
            .SetDestinationHeight((size.Height * zoom) as u32)
            .map_err(|e| anyhow!("设置高度失败: {:?}", e))?;

        page.RenderToStreamAsync(&stream)
            .map_err(|e| anyhow!("渲染失败: {:?}", e))?
            .get()
            .map_err(|e| anyhow!("等待渲染完成失败: {:?}", e))?;

        let size_u64 = stream
            .Size()
            .map_err(|e| anyhow!("获取流大小失败: {:?}", e))?;
        let mut buffer = vec![0u8; size_u64 as usize];

        let reader = windows::Storage::Streams::DataReader::CreateDataReader(&stream)
            .map_err(|e| anyhow!("创建数据读取器失败: {:?}", e))?;
        reader
            .LoadAsync(size_u64 as u32)
            .map_err(|e| anyhow!("加载数据失败: {:?}", e))?
            .get()
            .map_err(|e| anyhow!("等待数据加载失败: {:?}", e))?;
        reader
            .ReadBytes(&mut buffer)
            .map_err(|e| anyhow!("读取数据失败: {:?}", e))?;

        let img = image::load_from_memory(&buffer).map_err(|e| anyhow!("解码图像失败: {:?}", e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            rgba.as_raw(),
        ))
    }

    fn page_count(&self) -> usize {
        self.document
            .as_ref()
            .map(|d| d.PageCount().unwrap_or(0) as usize)
            .unwrap_or(0)
    }

    fn get_toc(&self) -> Vec<TocItem> {
        vec![]
    }
}

unsafe impl Send for WinPdfEngine {}
unsafe impl Sync for WinPdfEngine {}
