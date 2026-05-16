# 移除硬编码的个人路径，改为相对路径和环境变量

**日期**: 2026-05-16
**类型**: 重构

## 变更内容
- config.rs：移除所有 `D:/桌面/...` 和 `LLM_DIR_PLACEHOLDER...` 硬编码路径，改为相对路径和环境变量默认值
- ocr.rs：TESSDATA 常量改为 `TESSDATA_PREFIX` 环境变量；Tesseract 搜索路径优先环境变量 `TESSERACT_PATH`，其次相对路径，最后系统 PATH
- 删除无用的 `PROJECT_ROOT` 默认值

## 原因
- 个人路径写死在代码里，其他人 clone 后无法直接编译运行
- 跨平台可移植性要求

## 变更文件
- src/config.rs（所有路径默认值改为相对路径或环境变量）
- src/tools/ocr.rs（TESSDATA 硬编码 + Tesseract 搜索路径重构）
