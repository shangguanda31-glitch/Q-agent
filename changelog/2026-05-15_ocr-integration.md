# Tesseract OCR 集成

**日期**: 2026-05-15
**类型**: 功能新增

## 变更内容
- 实现 ocr_image 工具：Tesseract OCR 图片文字识别
- 支持中英文（chi_sim + eng 语言包）
- 自动搜索常见 Tesseract 安装路径
- 下载中文和英文训练数据到 tessdata/
- 图片消息不再尝试 LLM 视觉识别，直接保存到 image_cache/
- LLM 收到图片时自动注入 ocr_image 调用提示

## 新增文件
- tools/ocr.rs
- tessdata/chi_sim.traineddata, tessdata/eng.traineddata
