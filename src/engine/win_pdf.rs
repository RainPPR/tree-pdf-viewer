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
        let path_str = abs_path.to_string_lossy();

        // WinRT 的 PdfDocument::LoadFromFileAsync 需要异步环境，
        // 这里为了同步 Trait 接口，使用 block_on 或简单的同步包装。
        // 由于 windows-rs 的 async 处理稍显复杂，这里使用 StorageFile 先加载。
        let file =
            StorageFile::GetFileFromPathAsync(&windows::core::HSTRING::from(path_str.as_ref()))?
                .get()?;
        let doc = PdfDocument::LoadFromFileAsync(&file)?.get()?;

        self.document = Some(doc);
        Ok(())
    }

    fn render_page(&self, page_index: usize, zoom: f32) -> Result<ColorImage> {
        let doc = self
            .document
            .as_ref()
            .ok_or_else(|| anyhow!("文档未打开"))?;
        let page = doc.GetPage(page_index as u32)?;

        let stream = InMemoryRandomAccessStream::new()?;
        let options = PdfPageRenderOptions::new()?;

        // 设置渲染尺寸
        let size = page.Size()?;
        options.SetDestinationWidth((size.Width * zoom * 1.33) as u32)?;
        options.SetDestinationHeight((size.Height * zoom * 1.33) as u32)?;

        page.RenderToStreamAsync(&stream)?.get()?;

        // 从流中读取数据
        let size_u64 = stream.Size()?;
        let mut buffer = vec![0u8; size_u64 as usize];
        let reader = windows::Storage::Streams::DataReader::CreateDataReader(&stream)?;
        reader.LoadAsync(size_u64 as u32)?.get()?;
        reader.ReadBytes(&mut buffer)?;

        // 转换图像数据 (Windows.Data.Pdf 渲染出的是 BMP/PNG 编码的数据)
        // 使用 image crate 解码
        let img = image::load_from_memory(&buffer)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba.as_raw(),
        ))
    }

    fn page_count(&self) -> usize {
        self.document
            .as_ref()
            .map(|d| d.PageCount().unwrap_or(0) as usize)
            .unwrap_or(0)
    }

    fn get_toc(&self) -> Vec<TocItem> {
        // Windows.Data.Pdf 对目录的支持比较有限
        vec![]
    }
}

unsafe impl Send for WinPdfEngine {}
unsafe impl Sync for WinPdfEngine {}
