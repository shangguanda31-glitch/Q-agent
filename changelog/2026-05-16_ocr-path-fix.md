# 修复 OCR 搜索路径（从项目目录查找，不依赖 C 盘）

**日期**: 2026-05-16
**类型**: Bug 修复

## 变更内容
- OCR 优先从项目根目录 `tesseract/` 查找（`CARGO_MANIFEST_DIR`）
- 移除无用的 C 盘默认路径，精简搜索顺序

## 原因
- 从 `target/release/` 运行时相对路径找不到 tesseract.exe
- Tesseract 实际安装在项目目录下，不需要去 C 盘找

## 变更文件
- src/tools/ocr.rs（搜索路径重写）
