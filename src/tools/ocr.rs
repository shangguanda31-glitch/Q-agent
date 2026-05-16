use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;

use super::traits::{Tool, ToolResult};

fn tessdata_dir() -> String {
    std::env::var("TESSDATA_PREFIX").unwrap_or_else(|_| {
        format!("{}/tesseract/tessdata", env!("CARGO_MANIFEST_DIR"))
    })
}

pub struct OcrTool;

impl OcrTool {
    pub fn new() -> Arc<Self> { Arc::new(Self) }
    fn find_tesseract() -> Option<String> {
        // First check TESSERACT_PATH env var
        if let Ok(path) = std::env::var("TESSERACT_PATH") {
            if std::path::Path::new(&path).exists() { return Some(path); }
        }
        // Check common locations
        let project_root = env!("CARGO_MANIFEST_DIR");
        let candidates = vec![
            format!("{}/tesseract/tesseract.exe", project_root),
            format!("{}/tesseract/bin/tesseract.exe", project_root),
            "tesseract/tesseract.exe".to_string(),
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() { return Some(path.to_string()); }
        }
        // Try as PATH command
        if std::process::Command::new("tesseract").arg("--version").output().is_ok() {
            return Some("tesseract".to_string());
        }
        None
    }
}

#[async_trait]
impl Tool for OcrTool {
    fn name(&self) -> &str { "ocr_image" }

    fn description(&self) -> &str {
        "识别图片中的文字。用 Tesseract OCR 引擎提取文字，支持中英文。"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "image_path": {"type": "string", "description": "图片文件路径"}
            },
            "required": ["image_path"]
        })
    }

    async fn execute(&self, args: Value) -> ToolResult {
        let image_path = args.get("image_path").and_then(|v| v.as_str()).unwrap_or("");
        if image_path.is_empty() {
            return ToolResult::fail("需要 image_path 参数");
        }

        let exe = match Self::find_tesseract() {
            Some(p) => p,
            None => return ToolResult::fail(
                "Tesseract OCR 未安装。\n\
                 设置 TESSERACT_PATH 环境变量指向 tesseract.exe，或将其加入 PATH。\n\
                 下载: https://github.com/UB-Mannheim/tesseract/releases/latest"
            ),
        };

        let output = match Command::new(&exe)
            .args([image_path, "stdout", "-l", "chi_sim+eng", "--psm", "3"])
            .env("TESSDATA_PREFIX", tessdata_dir())
            .output().await
        {
            Ok(o) => o,
            Err(e) => return ToolResult::fail(format!("Tesseract 执行失败: {}", e)),
        };

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            return ToolResult::ok("图片中未识别到文字");
        }
        ToolResult::ok(text)
    }
}
