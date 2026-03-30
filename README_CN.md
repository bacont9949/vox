<div align="center">

# Vox

**语音输入，重新想象 —— 说任何语言，输出任何语言。**

基于 [Makepad](https://github.com/makepad/makepad) 和 Rust 构建的 macOS 菜单栏语音输入法。

按住 Option，说话，松开 —— 文字出现在光标处。

[English](README.md)

</div>

---

## 特性

- **按住即说** — 按住 Option 键录音，松开自动转录并输入文字
- **高质量语音识别** — 基于 [Qwen3-ASR](https://github.com/OminiX-ai/OminiX-MLX)（中文 CER 5.88，支持 30+ 语言）
- **LLM 智能优化** — 可选的后处理，修复识别错误、翻译、或转换文风
- **实时翻译** — 说中文，输出英文（或其他支持的语言）
- **文言文模式** — 说白话文，输出文言文
- **透明胶囊 UI** — 浮动状态指示器，带脉冲呼吸动画，全透明背景
- **多语言支持** — 简体中文、英语、日语、韩语、繁体中文、文言文
- **菜单栏应用** — 常驻菜单栏，无 Dock 图标（打包后）
- **隐私优先** — 音频通过 OminiX-MLX 本地处理，LLM 可配置（本地或云端）

## 架构

```
┌─────────────────────────────────────┐
│          Vox (Makepad 2.0)          │
│                                     │
│  菜单栏 ←→ 胶囊窗口 ←→ 设置窗口    │
│    ↕          ↕          ↕          │
│  CGEvent   音频 I/O     配置        │
│   Tap      (16kHz)   (~/.config)    │
└──────┬────────────┬─────────────────┘
       │            │
       ▼            ▼
  macos-sys     ominix-api
  (ObjC FFI)   (HTTP :18080)
                    │
              ┌─────┴─────┐
              │ Qwen3-ASR │  ← 本地 MLX 推理
              │ LLM (可选) │  ← 本地或云端 API
              └───────────┘
```

| Crate | 用途 |
|-------|------|
| `macos-sys` | macOS FFI — CGEvent tap、NSStatusBar、剪贴板、输入法切换、按键模拟 |
| `app` (vox) | Makepad 2.0 UI — 胶囊窗口、设置界面、音频捕获、HTTP 客户端 |

## 快速开始

### 前置条件

- macOS 14.0+，Apple Silicon（M1/M2/M3/M4）
- Rust 1.82+
- [OminiX-API](https://github.com/OminiX-ai/OminiX-API) 本地运行
- 终端需要辅助功能权限（系统设置 → 隐私与安全性 → 辅助功能）

### 1. 启动语音识别服务

```bash
cd /path/to/OminiX-API
PORT=18080 ASR_MODEL_DIR=~/.OminiX/models/qwen3-asr-0.6b cargo run --release
```

### 2. 运行 Vox

```bash
cd /path/to/vox
cargo run -p vox
```

### 3. 使用

1. 菜单栏找到 **MIC**
2. **按住左 Option 键**，开始说话
3. **松开** — 文字自动出现在光标位置

### 启用 LLM 优化（翻译/纠错）

```bash
MOONSHOT_API_KEY=sk-你的密钥 cargo run -p vox
```

也可以在设置界面配置任何 OpenAI 兼容的 API（点击 MIC → LLM Refinement → Settings）。

## 语言模式

| 菜单选项 | ASR 识别语言 | LLM 处理 | 示例 |
|---------|------------|---------|------|
| 简体中文 | 中文 | 纠正错别字 | 配森 → Python |
| English | 中文/英文 | 翻译为英文 | 你好 → Hello |
| 繁體中文 | 中文 | 转换为繁体 | 语音输入 → 語音輸入 |
| 日本語 | 日语 | 保持日文 | そのまま出力 |
| 한국어 | 韩语 | 保持韩文 | 그대로 출력 |
| 文言文 | 中文 | 转换为文言 | 今天天气好 → 今日天朗气清 |

> 翻译和文风转换需要启用 LLM Refinement（需要 API 密钥）。

## 配置

配置文件：`~/.config/vox/config.json`

### 环境变量

| 变量 | 说明 |
|------|------|
| `MOONSHOT_API_KEY` | Kimi API 密钥（自动配置 URL 和模型） |
| `VOICE_INPUT_LLM_API_KEY` | 任何 OpenAI 兼容的 API 密钥 |
| `VOICE_INPUT_LLM_API_URL` | LLM API 地址 |
| `VOICE_INPUT_LLM_MODEL` | LLM 模型名称 |
| `VOICE_INPUT_API_URL` | OminiX-API 地址（默认：`http://localhost:18080`） |

## 构建

```bash
# 开发模式
cargo run -p vox

# Release 构建
cargo build --release -p vox

# 打包 macOS .app（含 LSUIElement，无 Dock 图标）
make bundle
# 产物：target/Vox.app
```

## 技术栈

- **UI 框架**：[Makepad 2.0](https://github.com/makepad/makepad) — GPU 加速的原生 UI，Splash DSL
- **语音识别**：[Qwen3-ASR](https://github.com/OminiX-ai/OminiX-MLX) via OminiX-API — 纯 Rust MLX 推理
- **大语言模型**：任何 OpenAI 兼容 API（Kimi、DeepSeek、OpenAI、本地模型）
- **macOS 集成**：通过 `makepad_objc_sys` 的原生 ObjC FFI — CGEvent tap、NSStatusBar、NSPasteboard、TIS 输入法

## 许可证

MIT

## 致谢

- [Makepad](https://github.com/makepad/makepad) — UI 框架
- [OminiX-MLX](https://github.com/OminiX-ai/OminiX-MLX) — Apple Silicon 上的 ML 推理
- [OminiX-API](https://github.com/OminiX-ai/OminiX-API) — OpenAI 兼容的 API 服务
