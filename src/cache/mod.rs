use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// 管理 PDF 页面纹理的缓存
pub struct TextureCache {
    cache: LruCache<String, TextureHandle>,
}

impl TextureCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<TextureHandle> {
        self.cache.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, texture: TextureHandle) {
        self.cache.put(key, texture);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

pub type SharedTextureCache = Arc<Mutex<TextureCache>>;

/// 为 UI 线程准备的辅助函数，用于将 ColorImage 转换为 TextureHandle
#[allow(dead_code)]
pub fn load_texture(ctx: &Context, name: &str, image: ColorImage) -> TextureHandle {
    ctx.load_texture(name, image, TextureOptions::LINEAR)
}
