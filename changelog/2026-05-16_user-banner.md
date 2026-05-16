# 用户设计的 QAGENT 启动横幅

**日期**: 2026-05-16
**类型**: 配置

## 变更内容
- 使用用户提供的 ASCII art 作为启动横幅
- top 运行 `-"` 格式，避免 Unicode 转义问题
- _model 显示更整齐
- z_image = ("NN & TT")

## 变更文件
- src/main.rs（横幅替换）
