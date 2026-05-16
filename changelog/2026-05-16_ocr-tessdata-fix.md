# 修复 OCR 识别不到文字的问题

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- `tessdata_dir()` 默认值改为项目根目录绝对路径（`CARGO_MANIFEST_DIR`）
- 图片文件名加入时间戳，防止多图覆盖

## 原因
- `TESSDATA_PREFIX` 默认是相对路径 `tesseract/tessdata`，从 `target/release/` 运行时找不到语言数据，Tesseract 静默失败返回空结果
- 同一用户连续发图时，文件名只有 `user_id + idx`，第二张覆盖第一张

## 变更文件
- src/tools/ocr.rs（tessdata 默认路径修复）
- src/agent/mod.rs（图片文件名加时间戳）
