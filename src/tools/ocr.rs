use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;

use super::traits::{Tool, ToolResult};

const TESSDATA: &str = "PROJECT_ROOT_PLACEHOLDER/qq-assistant/tesseract/tessdata";

pub struct OcrTool;

impl OcrTool {
    pub fn new() -> Arc<Self> { Arc::new(Self) }
    fn find_tesseract() -> Option<String> {
        for path in &[
            "D:/tesseract/tesseract.exe",
            "PROJECT_ROOT_PLACEHOLDER/qq-assistant/tesseract/tesseract.exe",
            "PROJECT_ROOT_PLACEHOLDER/qq-assistant/tesseract/bin/tesseract.exe",
            "C:/Program Files/Tesseract-OCR/tesseract.exe",
            "C:/Program Files (x86)/Tesseract-OCR/tesseract.exe",
        ] {
            if std::path::Path::new(path).exists() { return Some(path.to_string()); }
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
                "Tesseract OCR 未安装。请手动安装：\n\
                 1. 下载 https://github.com/UB-Mannheim/tesseract/releases/latest\n\
                 2. 安装时勾选中文语言包\n\
                 或直接运行: D:\\桌面\\编程作品\\Sandy ONE\\qq-assistant\\tesseract\\tesseract_setup.exe"
            ),
        };

        let output = match Command::new(&exe)
            .args([image_path, "stdout", "-l", "chi_sim+eng", "--psm", "3"])
            .env("TESSDATA_PREFIX", TESSDATA)
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
