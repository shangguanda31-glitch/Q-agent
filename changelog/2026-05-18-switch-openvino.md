# 切换 OpenVINO 推理后端

**日期**: 2026-05-18
**类型**: 配置变更

## 变更内容
- `.env`: 添加 `LLM_URL=http://127.0.0.1:8000` 指向 OpenVINO 后端
- `.env`: 添加 `EMBED_URL=http://127.0.0.1:8082` 显式指定嵌入服务
- `target/release/.env`: 同步更新
- `.env.example`: 更新注释说明两个后端选择
- `src/main.rs`: `start_llama_server` 增加对 `cfg.llm_url` 的检查，当已配置的 LLM_URL 指向运行中的服务时直接使用，不再覆盖

## 影响范围
- LLM 推理从 llama.cpp (CUDA, :8081) 切换到 OpenVINO (Intel GPU, :8000)
- Embedding 服务不变，仍由 llama.cpp 在 :8082 提供

## 切换理由
OpenVINO INT4 方案在 Intel GPU 上提供接近的推理性能，同时释放 NVIDIA GPU 显存