use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;

fn main() -> anyhow::Result<()> {
    // 只有在 Windows 平台才执行此逻辑
    #[cfg(target_os = "windows")]
    {
        setup_pdfium()?;
    }
    Ok(())
}

fn setup_pdfium() -> anyhow::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let target_dir = Path::new(&manifest_dir).join("target").join(env::var("PROFILE")?);
    
    // 我们希望把 dll 放到可执行文件所在目录
    let dll_path = target_dir.join("pdfium.dll");
    
    if !dll_path.exists() {
        println!("cargo:warning=PDFium DLL missing, attempting to download...");
        
        let url = "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-win-x64.tgz";
        let response = reqwest::blocking::get(url)?;
        
        if !response.status().is_success() {
            anyhow::bail!("下载 PDFium 失败: {} - {}", response.status(), url);
        }

        let bytes = response.bytes()?;
        let tar_gz = GzDecoder::new(&bytes[..]);
        let mut archive = Archive::new(tar_gz);
        
        // 确保输出目录存在
        fs::create_dir_all(&target_dir)?;

        let mut found = false;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            
            // 在 .tgz 中，pdfium.dll 通常位于 bin/pdfium.dll 或直接在根目录
            if path.file_name().and_then(|s| s.to_str()) == Some("pdfium.dll") {
                let mut dll_content = Vec::new();
                entry.read_to_end(&mut dll_content)?;
                fs::write(&dll_path, dll_content)?;
                println!("cargo:warning=PDFium DLL extracted to {:?}", dll_path);
                found = true;
                break;
            }
        }
        
        if !found {
            anyhow::bail!("在下载的压缩包中未找到 pdfium.dll");
        }
    }
    
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
