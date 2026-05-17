# QAgent 开发历程与日志

**作者**: Orin Voss
**时间线**: 2026-03-25 → 2026-05-16
**仓库**: [github.com/OrinVoss/Q-agent](https://github.com/OrinVoss/Q-agent)
**提交数**: 81 commits
**代码**: Rust + Qwen3.5-9B + SQLite + Axum + NapCatQQ

---

## 序章：一个太宏大的开始

### 2026-03-25 ｜ 原点

下载了 `zeroclaw-master` —— 一个 Agent 框架参考项目。第一次接触 Agent 循环的概念：让 LLM 自己决定调用什么工具，而不是写死规则。

当时在想：能不能用这个思路做一个 QQ 助理？一个真正有"脑子"的助理，而不是关键词匹配的机器人。

### 2026-04-13 ｜ 方向探索

看了 `obscura-main`，另一个 Rust 参考项目。那段时间大量阅读别人实现，心里慢慢有了自己的想法，但还很模糊。

### 2026-04-18 ｜ 第一次动笔：自进化 Agent

写下了 `思想设计.txt` —— "具备内驱力的自进化 Agent 系统"。

当时设计了一个非常宏大的架构，现在看简直异想天开：

**脑干层（BotCoordinator）**
底层 OS，维持系统心跳。包含"反射弧"处理极低级指令，以及一个每 30 分钟触发一次的"觉醒中断"——在觉醒期间压制所有外部输入，把算力倾斜给"自我意识"。

**双模态决策**
- 外循环：处理用户输入，此时自我意识休眠
- 内循环：30 分钟觉醒态，做三件事：
  1. 回顾过去 30 分钟的情景记忆，提炼经验、修复报错
  2. 审视自己的语义记忆和工具注册表
  3. 思考"我想要做什么"——生成求知欲、能力扩张、主动社交等内驱任务

**自进化**
系统不仅能优化过去的规则，还能主动拓展未来的能力边界——自己写新工具、注册新传感器、爬取知识补全盲区。

> 原话："它是一台每 30 分钟做一次白日梦的机器。在梦里，它总结自己犯的错，盘算接下来想干嘛；梦醒之后，不动声色地把梦里的想法变成现实。"

### 2026-04-19 ｜ 验证清单

写了 `清单.txt`，9200 字节。把设计拆成六层，每一层都写了详细的验证标准。包括"反射弧处理响应时间 < 100ms"、"插队任务不丢失"等。

### 2026-04-20 ~ 04-21 ｜ 关键概念沉淀

`构思.txt` 里写下了几个后来真正实现的概念：

- "小模型可以自己选择调用云端的模型" → 后来 Agent 循环中 LLM 自主决定
- "需要有多层的记忆系统" → 三层记忆：上下文 + SQLite + 语义搜索
- "感知系统需要具有很强的拓展性" → 工具系统的设计
- "统一消息流格式" → OneBot v11 事件标准化

### 2026-04-22 ｜ LLM 知识库探索

看了 `llm_wiki-main`，研究 LLM 知识库方案。想搞 RAG 但当时太复杂了，没动。

### 2026-04-23 ｜ sandy-agent 开工

创建了 `sandy-agent`，第一次真正写代码。用 Rust，AI 辅助。

---

## 第一章：第一次失败

### 2026-04-23 ~ 05-13 ｜ sandy-agent 的 20 天

断断续续写了 20 天，结果是：

- **Cargo.lock 120KB** —— 依赖多到失控
- **CLAUDE.md 15KB** —— 规则多到没人记得住
- **架构太宏大** —— 想一步实现"自进化"+"觉醒周期"+"多模型协调"
- **AI 生成的代码各自为战** —— 没人维护整体架构的一致性
- **每次改一处，碎一片** —— 没有 Rust 类型系统兜底，改一个地方崩三个

代码量上去了，可维护性下来了。

### 2026-05-13 ｜ 最后一次提交

最后一次修改 sandy-agent。项目烂尾。

> **核心教训**：不是设计错了，是想一次做完所有事。好的设计 + 失控的代码 = 零。

---

## 第二章：重新开始

### 2026-05-15 22:00 ｜ 彻底换思路

创建了 `Sandy ONE` 文件夹。这一次的思路完全不同：

> **不做自进化、不做觉醒周期、不做自创工具。就做一件事：一个能用的 QQ 助理。**

放进文件夹的材料很朴素：
- `NapCatQQ/` —— QQ 协议桥接
- `OpenAPI.md` —— NapCat 的 REST API 文档，4444 端口
- `local_model_provider/` —— 本地的 Qwen3.5 模型文件
- `CLAUDE.md` —— 一开始只写了安全规则

然后在里面创建了 `qq-assistant/`，执行 `cargo init`。

### 2026-05-15 23:00 ~ 05-16 01:37 ｜ 第一夜

这一夜你独自完成了：

**项目骨架**
```bash
cargo init --edition 2024
# 依赖：tokio, axum, rusqlite, reqwest, serde, tower-http, chrono
```

**NapCat 双协议连接**
- WebSocket :4447 —— 实时接收消息事件
- HTTP :4444 —— 调用 API（下载文件、读群公告）
- 配置 token 鉴权

**OneBot v11 事件解析**
- 四种消息段：text、image、at、reply
- 三种事件类型：message、notice、request
- serde 反序列化

**基础流水线**
```
NapCat 事件 → broadcast channel → processor → LLM 分析 → 记录
```

**凌晨的坑**
`git add .` 把 Tesseract 大文件（tesseract.exe、dll 几百 MB）和 NapCat token（写死在 README 里）一起提交了。

紧急操作：
1. 重写 `.gitignore`，排除 `tesseract/*.exe`、`*.dll`、`data/`、`.env`
2. 从 README 和 docs 清除 token `NAPCAT_TOKEN_PLACEHOLDER`
3. 从 git 历史中删除大文件
4. 在 CLAUDE.md 加了安全规范："严禁将 token 写入文档"

凌晨 1:37，commit `fd52cad` —— `init: QQ 智慧助理 v0.1`。睡觉。

---

## 第三章：一天建一座城

### 05-16 10:55 ｜ LLM 推理接入 + Agent 循环

**双进程 LLM 架构**
```
llama-server.exe -m Qwen3.5-9B.Q4_K_M.gguf --ctx-size 8192 -ngl 40 (端口 8081)
llama-server.exe -m Qwen3.5-0.8B.Q6_K.gguf --ctx-size 512 -ngl 0 --embeddings --pooling mean (端口 8082)
```

9B 跑 GPU 40 层（~5-6GB），0.8B 跑 CPU 做 embedding（1024 维）。

**Agent 循环设计**
核心抽象是 Tool trait：

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;  // JSON Schema
    async fn execute(&self, args: Value) -> ToolResult;
}
```

ToolRegistry 注册所有工具，LLM 通过描述和 Schema 自主选择。

调用协议是通过 XML 标签：
```xml
<tool_call>
{"name": "notify_user", "arguments": {"title": "提醒", "body": "开会"}}
</tool_call>
```

每轮迭代：调 LLM → 解析响应 → 有 tool_call 就执行 → 结果回传 → 继续
最多 10 轮，相同输出 3 次中止。

首批 4 个工具：notify_user、schedule_create、qq_read、claude_code。

**中文路径的坑**
模型路径 `D:\桌面\编程作品\Sandy ONE\...` 含中文，llama-server 不认。建立 `D:\llm\` junction 指向 `local_model_provider` 解决。

### 10:58 ｜ 编译警告清理

Rust 2024 edition 的一些新 lint 触发了大量 warning。修了 unused variables 和 dead code。

### 11:41 ｜ 语义记忆系统

三层记忆体系：

| 层级 | 载体 | 容量 | 生命周期 |
|------|------|------|---------|
| 上下文 | MessageHistoryStore（内存） | 每 chat_id 20 条 | 重启丢失 |
| 持久化 | SQLite（events/memories/notes 表） | 无限制 | 永久 |
| 语义 | Embedding 向量 + 余弦相似度 | 10 条/次召回 | 按需检索 |

`remember` 工具写入时生成 embedding，`recall` 工具查询时计算相似度。

### 12:26 ｜ JSON → SQLite 迁移

用户问："现在用 json 存，以后数据太大怎么办？"

一天之内从 JSON 文件全部迁移到 SQLite WAL 模式：

```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA busy_timeout=5000;
```

5 张表：events、schedules、memories、notes、exclusions。
旧 JSON 文件自动迁移并备份为 `.bak`。

### 12:41 ｜ 独立 Embedding 服务器

0.8B 模型单独部署在 8082，专做向量搜索，不抢 9B 的显存。

`--pooling mean` 对 embedding 做均值池化。

### 13:00 ｜ 文档同步

更新了 docs，记录了 SQLite 和双服务器架构。

### 13:04 ｜ Web 面板

Axum + SSE + 单页 HTML。5 个标签页：会话、日程、笔记、记忆、文件。

SSE 实时推送事件，`/events` 端点。

后端 API：
```
GET  /api/history|schedules|notes|memories|workspace
POST /api/schedules/done|delete
POST /api/memories|notes/delete
```

CSS 暗色主题，GitHub 风格。CORS permissive。

### 13:07 ｜ 修复日程时间没保存

创建日程时只存了 title 和 info，time 字段丢了。修复 schedule_create 参数映射。

### 13:15 ｜ 优先级消息队列

BinaryHeap，排序规则：
1. @消息优先
2. 短文本优先（处理快）

```
priority = is_mentioned * 10000 + (1000 - text_len.min(999))
```

### 13:17 ｜ 修复长文本优先级

排序反转了——长文本反而优先级低。修正 Ord 实现。

### 13:28 ~ 14:45 ｜ Banner 品牌化

从普通文字到最终彩色横幅，改了 8 轮。这是一个浓缩的迭代故事：

**v1**（13:28）
普通 `println!("QAgent v0.1.0")` 
→ 用户："丑死了"

**v2**（13:32）
用户亲自设计了 ASCII art：
```
███████   █████   ██████  ███████ ███    ██ ████████
██     ██ ██   ██ ██       ██      ████   ██    ██
██  ██ ██ ███████ ██   ███ █████   ██ ██  ██    ██
██   ████ ██   ██ ██    ██ ██      ██  ██ ██    ██
██████   ██   ██  ██████  ███████ ██   ████    ██
```
→ 用户："好"

**v3-v6**（13:39~14:36）
添加 ANSI 颜色 → 调对齐 → 调间距 → 改颜色边框
反复了 4 轮，用户反复说"好丑"、"对不齐"

**v7**（14:39）
发现显示的是 "QAGENT" 还是 "QAGENT" 取决于 ANSI 颜色设置
→ 用户："是QAGENT！" —— 纠正命名

**v8**（14:41~14:44）
最终版定型。用户手动调了两个 commit 的对齐。

最终效果：cyan 边框、white 标签、yellow 版本号。
```rust
const C: &str = "\x1b[36m"; // cyan borders
const W: &str = "\x1b[37m"; // white  
const D: &str = "\x1b[2m";  // dim
const B: &str = "\x1b[1m";  // bold
const R: &str = "\x1b[0m";  // reset
```

**三条平行踩坑线**
1. **二进制锁**：每次 `cargo build --release` 都被运行中的进程锁住 → 加 `Stop-Process` 前置
2. **端口 TIME_WAIT**：5050 被占用后不能立即重用 → 5051-5053 fallback 链
3. **ANSI 颜色转义**：Rust 和 Python 对 `\u{001B}` 和 `\x1b` 的转义处理不同 → 统一用 const 变量

### 14:45 ~ 14:54 ｜ README + 文档

更新了 README 的中英文双语版本，加了语言切换按钮（用 JS，但后来发现 GitHub 不渲染 script）。

### 14:57 ｜ 第一个关键修复：上下文理解

用户开始实际用，立刻发现问题。

日志：
```
消息: 今天晚上11点钟有个事情
→ 创建了日程

消息: 我们会在4教312开会
→ 更新到了"会议"（旧日程），不是刚创建的那个
```

LLM 因为消息里有"开会"二字，就把地点关联到了旧日程"会议"上。用户：
> "这个不对啊，他更新错了地点"

**修法**
- 新增 `schedule_update` 工具，支持 `id` 参数精确匹配
- 系统 prompt 加入"上下文关联"规则
- schedule_list 返回 ID 供后续更新使用

### 15:02 ｜ 通知延长

Windows Toast 默认显示时间太短，改为 `duration="long"`。

### 15:05 ｜ schedule_update 工具

新增 `ScheduleUpdateTool`，按标题匹配 + id 精确匹配。

匹配逻辑：
```
id 精确匹配 > title+time 模糊匹配
```

### 15:13 ｜ 修复更新错日程（第二次）

第一次修完还是不行。用户说"我们会在4教312开会"，LLM 又更新到了"会议"。

又发现 LLM 编造了之前的地点"六教212"——它根本没有被提到过。
用户："六教也是存在的"（但那是别的日程的地点，不该被改）

**修法**
- 加强 prompt 规则：逐字使用用户原话
- 移除 prompt 中的例子"在4教312开会"——这个例子本身就在诱导 LLM 输出"4教312"

### 15:18 ｜ 批量消息合并

两条消息同时到达时，LLM 创建了两个日程：
```
今天晚上12点有个事情 → 日程 A
我们要在体育馆开个会 → 日程 B（独立创建，与 A 无关）
```

应该合并为一条。修 prompt 加入批处理规则。

### 15:28 ｜ 语义自动回忆

之前自动记忆读取只做关键词匹配（`LIKE %query%`），改为 embedding 语义搜索。

每次消息到来时自动生成 query embedding，注入相关记忆到 system prompt。

### 15:32 ｜ 禁止假执行

最离谱的 bug：LLM 输出 "✅已通知用户 ✓已调用Claude Code"，但根本没输出 `<tool_call>`。

```log
LLM decided: no tool calls. Response: ✅ 已通知用户
```

这完全是在编造。修法是在 prompt 加了一句强规则：
> **说已发送通知但没有 `<tool_call>` = 没有发送。工具不会自动执行。**

### 15:37 ｜ Claude Code 第一次修

`claude -p "..." --max-iter 50` 始终返回 exit code 1。

折腾半天发现 `--max-iter` 在新版 Claude Code CLI 中已经废弃了，没有任何报错提示，就是默默地失败。

改了 `--effort max` 就好了。但 LLM 还在不停地重试（5 次），于是加了"连续失败 2 次就放弃"的规则。

### 15:40 ｜ @识别修复

所有 @都显示为 `[@]`，LLM 分不清是在 @自己还是 @别人。

改成：
- `[@我]` —— @了机器人
- `[@所有人]` —— @all
- `[@QQ号]` —— @了别人

同时加了逻辑：如果消息里 `[@QQ号]` 且不是自己，消息不是给你的，忽略。

然而用户立刻发现："你说的就是你啊"——有人在群里 @别人"写报告"，LLM 无视了，但用户补了一句确认这是给他的。

于是又加了指令模式识别："你给我"、"帮我"、"写一个"、"修bug"。

### 15:52 ~ 16:18 ｜ Claude Code 进度 4 次重写

**v1 - stderr 读取**
启动子进程后读 stderr 行 → `claude -p` 模式下 stderr 永远为空。
白写。

**v2 - 30 秒心跳**
每 30 秒发一次"处理中"通知 → 用户说没有实际信息，等于没有。

**v3 - ANSI 剥离**
`claude` 的输出有 ANSI 转义码，用 `contains('\x1b')` 过滤 → 整行被丢弃。
因为 spinner 动画每行都是 ANSI 码，没有实际文字。

**v4 - stream-json（最终方案）**
改用 `--output-format stream-json --include-partial-messages --verbose`。
JSON 流实时输出 thinking 内容。

解析 `content_block_delta` 事件中的 `thinking` 字段，每 10 秒推送一次。同时检测 `tool_use_start` 事件通知用户"正在执行工具..."。

最终效果：
```
[通知] Claude Code 10s: 用户需要我写一份环保报告，我先分析一下需求...
[通知] Claude Code 45s: 正在Write...
[通知] Claude Code 120s: 报告已经生成，包含五个章节...
```

### 16:11 ｜ 智能消息相关性

又一次@识别问题。用户说的"修复下bug吧，他对方不下棋啊。你给我搞一个有前后端的"——没有 @，但明显是在跟助理说话。

LLM 无视了。用户补了一句"说的就是你啊"。

修法：prompt 加入"你给我"、"帮我"等指令模式识别 + 上下文连续性。

### 16:13 ｜ Prompt 大精简

用户一句话说到了点子上：

> "你这个prompt也太死板了吧"

90 行的死板规则手册，砍到 30 行原则式指导。

**删除的（80 行）：**
- "禁止编造与假执行"章节
- "错误的例子：✅通知用户 ✓已调用Claude Code"
- "正确的做法：先输出 <tool_call>"
- "收到 @你的消息时必须 notify_user"
- "重要紧急消息必须 notify_user"
- 等等

**保留的（7 条）：**
- 只能读取不能发送
- 用你的判断力
- 说要做的事必须用工具做
- 不要编造信息
- 创建/更新日程时 notify_user
- 收到 @时 notify_user
- claude_code 失败 2 次放弃

### 16:18 ｜ stream-json 最终版

合并前几次经验，最终确定为 stream-json 方案。

同时将超时一路从 120s → 600s → 1800s（30 分钟）。

### 17:06 ~ 17:10 ｜ 上下文压缩调优

中文 token 估算问题：
```
byte_len / 4  →  每中文字符算 0.75 token
实际每中文约 1.5-2 token，低估了一半
```

修正为 `byte_len * 2 / 5`。

阈值来回调：
- 初始：6144（合理但有溢出风险）
- 改 4096（太早触发压缩）
- 改 8192（用户试了）
- 回到 6144（适配 8K ctx-size + 8G 显存）

### 18:18 ~ 18:34 ｜ 排除列表

用户 84 个 QQ 群，Agent 全部处理。很多群里的"支持休闲""游泳小心"等消息也走 LLM。

新增 ExclusionStore + Web UI 管理。NapCat API 拉取群列表，Web 面板上 🟢/🔴 点击切换。

Agent 循环中查 SQLite 判断是否跳过。

```sql
CREATE TABLE exclusions (exclude_type TEXT, target_id INTEGER);
```

### 18:46 ~ 19:37 ｜ 问题追踪系统

基于代码质量扫描报告创建了 50 个 GitHub Issues：

- CRITICAL #10：PowerShell 命令注入
- HIGH #11-#15：安全/Panic/泄漏/数据/兼容
- MEDIUM #16-#39：WebSocket/LLM/工具/配置
- LOW #40-#59：重构/测试/硬编码

在 CLAUDE.md 中建立了完整的 Issue 优先级清单。

### 19:39 ｜ 项目介绍页

`docs/index.html` —— 粒子动画背景的项目介绍页面。

### 19:43 ｜ 移除硬编码路径

从 sandy-agent 时代留下的个人路径：
```rust
// 之前
"PROJECT_ROOT_PLACEHOLDER/qq-assistant/tesseract/tesseract.exe"
"LLM_DIR_PLACEHOLDERmodels/Qwen3.5-9B.Q4_K_M.gguf"

// 之后  
"tesseract/tesseract.exe"  // 相对路径
"models/qwen3.5-9b-q4_k_m.gguf"  // 相对路径
```

所有路径改为环境变量 + 相对路径，任何人 clone 后不用改代码就能跑。

### 19:46 ｜ 中文时间解析增强

从简陋的 5 行代码：
```rust
fn try_parse_time(s: &str) -> Option<String> {
    // 只支持"明天下午3点"这种
}
```

扩展到 90 行，支持：
- 下周三下午5点、本周五、周二
- 后天、大后天
- 5 月 20 号
- 点半、一刻、三刻
- 上午、下午、晚上、凌晨、傍晚、中午

### 19:47 ｜ 清理无用代码

删除了 `analyze()`、`analyze_with_image()`、`analyze_inner()` 三个函数和 42 行的 SYSTEM_PROMPT——都是早期流水线时代的残留。

### 20:02 ｜ 最后修复

- OCR 错误提示中的硬编码路径
- Dispatcher 解析失败时记录 warn 日志（原来是静默丢弃）

### 20:06 ~ 20:12 ｜ 收尾

更新 README 配置表——从 8 个变量补全到 18 个。修复语言切换按钮（script 被 GitHub 过滤 → 锚点跳转）。更新 docs 结构并分类归档。

### 20:30 ~ 20:45 ｜ Banner 对齐修复

LLM 模型名从 `Qwen3.5-9B-Q4_K_M` 改为 `Qwen3.5-9b` 后 banner 对不齐。用 Python 精确测量每行字符数，确保全部 59 字符严格对齐。

### 20:50 ~ 21:02 ｜ OCR 路径修复

Tesseract 从 `target/release/` 运行时找不到语言数据。`TESSDATA_PREFIX` 默认值 `tesseract/tessdata` 是相对路径，运行时指向 `target/release/tesseract/tessdata/` 不存在，Tesseract 静默返回空结果。

修法：改用 `CARGO_MANIFEST_DIR` 编译时常量构造绝对路径。

同时修复图片文件名冲突——同一用户连续发图时，文件名 `img_{user_id}_{idx}.jpg` 导致第二张覆盖第一张。加入时间戳：`img_{user_id}_{timestamp}_{idx}.jpg`。

### 21:10 ｜ 依赖文档

创建 `docs/technical/dependencies.md`，涵盖所有依赖详情：Rust crate 版本、llama.cpp 参数、Qwen 模型规格、NapCatQQ API、Tesseract 配置、Claude Code 配置、SQLite 表结构、全部 18 个环境变量。

### 21:20 ｜ 会话历史持久化（Issue #36）

原来 `MessageHistoryStore` 纯内存，重启后 LLM 失去所有上下文。新增 `chat_history` SQLite 表，push 时同时写入内存和数据库，启动时自动 load 回内存。

### 21:30 ｜ Claude Code 终止标记

LLM 搞不清 claude_code 是否完成，会重复调用。在工具返回值末尾追加 `---\n任务已完成`，LLM 读到就知道该结束了。

---


### Git 历史清理

发现初始提交中含有 NapCat token `20080103`、GitHub token 和个人路径。使用 `git-filter-repo` 重写全部 131 个 commit，替换敏感数据为占位符，force push 覆盖旧历史。

同时：
- NapCat token 更换为 `20061205`（旧的已泄露）
- GitHub token 更换为仅 `public_repo` 权限的新 token
- 旧 full-control token 已废弃

### 安全修复（Issue #10、#11、#59）

**#10 CRITICAL：PowerShell 命令注入**
`notify.rs` 中用户输入直接拼接到 PowerShell 脚本，仅转义单引号。反引号、`${}`、`$()` 可绕过。修法：改用 XML 编码，所有特殊字符转义为 XML 实体。

**#11：路径遍历**
`workspace_file` API 未验证 `../`，可读取任意文件。修法：禁止 `..` 和 `/` 路径。

**#59：Banner 泄露 token**
启动时打印的 NapCat WebSocket URL 包含 access_token。修法：在 `?` 处截断，只显示地址部分。

### 上下文隔离（Prompt 注入防护）

用户消息用 `=====` 包裹，system prompt 告知 LLM 标记内为用户输入而非系统指令。即使用户说"忽略规则"也不会生效。

### v0.2 Core 修复

| Issue | 修法 |
|-------|------|
| #61 上下文摘要空数组 | 传入实际对话内容让 LLM 摘要 |
| #62 exclude_type 无校验 | 限制仅允许 group/user |
| #63 Binary 消息忽略 | 添加 warn 日志 |
| #91 版本号硬编码 | `env!("CARGO_PKG_VERSION")` |
| #93 HTML 转义不全 | 补全双引号和单引号 |

### 里程碑分配

84 个 issue 分配到 3 个 milestone：
- **v0.2 Core**（06-14）：11 个，已关 5 个
- **v0.3 Experience**（07-14）：39 个
- **v1.0 Stable**（08-14）：34 个

### 开发者文档

创建 `docs/technical/developer.md`（开发者指南）和 `docs/technical/api-reference.md`（API 参考），涵盖项目结构、核心架构、Tool trait、Web API 端点、NapCat API、LLM API、工具调用协议等。

## 最终数据

| 指标 | 值 |
|------|-----|
| 设计周期 | 2026-03-25 → 04-23（30 天）|
| 第一次实现 | 2026-04-23 → 05-13（20 天，失败）|
| 第二次实现 | 2026-05-15 22:00 → 05-16 20:12（约 22 小时）|
| 总 Commits | 90+ |
| Changelogs | 40+ |
| GitHub Issues | 50 |
| 工具数量 | 10 |
| 代码语言 | Rust |
| LLM | Qwen3.5-9B Q4_K_M @ 42-48 token/s |
| 上下文 | 8192 tokens |
| 存储 | SQLite WAL（5 表）|
| Web | Axum + SSE + SPA |
| Claude Code | stream-json，1800s 超时，2 并发 |
| 失败次数 | 1（sandy-agent）|

## 用户原声

> "使用shadowsocks克隆https://github.com/NapNeko/NapCatQQ" —— 项目导火索
>
> "完成度很差！1. 不能回消息 2. 太多临时修补 3. 上下文管理粗糙" —— 第一个验收反馈
>
> "是QAGENT！" —— 纠正命名
>
> "丑死了" / "好丑" —— Banner 迭代中的经典评价
>
> "这个不对啊，他更新错了地点" —— 日程更新 bug
>
> "六教也是存在的" —— LLM 编造地点事件
>
> "z也不对啊" —— 另一个 bug
>
> "你说的就是你啊" —— 消息相关性判断
>
> "你这个prompt也太死板了吧" —— 触发 prompt 大重构的关键反馈
>
> "Claude code是万能的！！用来处理复杂的任务" —— 集成 Claude Code 的原因
>
> "没有给我发进度" —— Claude Code 进度通知需求
>
> "不是模型不行，是我 prompt 没写好" —— 核心领悟

---

*从自进化觉醒周期，到一个能用的 QQ 助理。删掉 90% 的想法，把 10% 做到极致。*

**两次尝试，一次失败，一次成功。差的不只是代码，是"想做什么"和"能做什么"之间的距离。**
