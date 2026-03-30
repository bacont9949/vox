<div align="center">

# Vox

**Voice input reimagined — speak in any language, type in any language.**

A macOS menu-bar voice input app built with [Makepad](https://github.com/makepad/makepad) and Rust.

Press Option, speak, release — your words appear wherever the cursor is.

[中文文档](README_CN.md)

</div>

---

## Features

- **Press-to-talk** — Hold Option key to record, release to transcribe and inject text
- **High-quality ASR** — Powered by [Qwen3-ASR](https://github.com/OminiX-ai/OminiX-MLX) (CER 5.88 on Chinese, 30+ languages)
- **LLM Refinement** — Optional post-processing to fix recognition errors, translate, or convert text style
- **Real-time translation** — Speak Chinese, output English (or any supported language)
- **Classical Chinese mode** — Speak modern Chinese, output 文言文
- **Transparent capsule UI** — Floating status indicator with pulse animation, fully transparent background
- **Multi-language** — Chinese, English, Japanese, Korean, Traditional Chinese, Classical Chinese
- **Menu bar app** — Lives in your menu bar, no Dock icon (when bundled)
- **Privacy first** — Audio processed locally via OminiX-MLX, LLM refinement configurable (local or cloud)

## Architecture

```
┌─────────────────────────────────────┐
│            Vox (Makepad 2.0)        │
│                                     │
│  Menu Bar ←→ Capsule ←→ Settings   │
│       ↕           ↕         ↕       │
│   CGEvent    Audio I/O    Config    │
│     Tap      (16kHz)    (~/.config) │
└──────┬────────────┬─────────────────┘
       │            │
       ▼            ▼
  macos-sys     ominix-api
  (ObjC FFI)   (HTTP :18080)
                    │
              ┌─────┴─────┐
              │ Qwen3-ASR │  ← local MLX inference
              │ LLM (opt) │  ← local or cloud API
              └───────────┘
```

| Crate | Purpose |
|-------|---------|
| `macos-sys` | macOS FFI — CGEvent tap, NSStatusBar, clipboard, input source, key simulation |
| `app` (vox) | Makepad 2.0 UI — capsule window, settings, audio capture, HTTP client |

## Quick Start

### Prerequisites

- macOS 14.0+ on Apple Silicon (M1/M2/M3/M4)
- Rust 1.82+
- [OminiX-API](https://github.com/OminiX-ai/OminiX-API) running locally
- Accessibility permission for the terminal (System Settings → Privacy → Accessibility)

### 1. Start the ASR service

```bash
cd /path/to/OminiX-API
PORT=18080 ASR_MODEL_DIR=~/.OminiX/models/qwen3-asr-0.6b cargo run --release
```

### 2. Run Vox

```bash
cd /path/to/vox
cargo run -p vox
```

### 3. Use it

1. Look for **MIC** in the menu bar
2. **Hold left Option key** and speak
3. **Release** — text appears at your cursor

### With LLM Refinement (translation/correction)

```bash
MOONSHOT_API_KEY=sk-your-key cargo run -p vox
```

Or configure any OpenAI-compatible API in Settings (MIC → LLM Refinement → Settings).

## Language Modes

| Menu Selection | ASR Language | LLM Action | Example |
|---------------|-------------|------------|---------|
| 简体中文 | Chinese | Correct typos | 配森 → Python |
| English | Chinese/English | Translate to English | 你好 → Hello |
| 繁體中文 | Chinese | Convert to Traditional | 语音输入 → 語音輸入 |
| 日本語 | Japanese | Preserve Japanese | そのまま出力 |
| 한국어 | Korean | Preserve Korean | 그대로 출력 |
| 文言文 | Chinese | Convert to Classical | 今天天气好 → 今日天朗气清 |

> Translation and style conversion require LLM Refinement enabled (needs API key).

## Configuration

Config file: `~/.config/vox/config.json`

### Environment Variables

| Variable | Description |
|----------|-------------|
| `MOONSHOT_API_KEY` | Kimi API key (auto-configures URL and model) |
| `VOICE_INPUT_LLM_API_KEY` | Any OpenAI-compatible API key |
| `VOICE_INPUT_LLM_API_URL` | LLM API base URL |
| `VOICE_INPUT_LLM_MODEL` | LLM model name |
| `VOICE_INPUT_API_URL` | OminiX-API base URL (default: `http://localhost:18080`) |

## Build

```bash
# Development
cargo run -p vox

# Release
cargo build --release -p vox

# macOS .app bundle (with LSUIElement — no Dock icon)
make bundle
# Output: target/Vox.app
```

## Tech Stack

- **UI Framework**: [Makepad 2.0](https://github.com/makepad/makepad) — GPU-accelerated native UI with Splash DSL
- **ASR Engine**: [Qwen3-ASR](https://github.com/OminiX-ai/OminiX-MLX) via OminiX-API — pure Rust MLX inference
- **LLM**: Any OpenAI-compatible API (Kimi, DeepSeek, OpenAI, local models)
- **macOS Integration**: Raw ObjC FFI via `makepad_objc_sys` — CGEvent tap, NSStatusBar, NSPasteboard, TIS input sources

## License

MIT

## Credits

- [Makepad](https://github.com/makepad/makepad) — The UI framework
- [OminiX-MLX](https://github.com/OminiX-ai/OminiX-MLX) — ML inference on Apple Silicon
- [OminiX-API](https://github.com/OminiX-ai/OminiX-API) — OpenAI-compatible API server
