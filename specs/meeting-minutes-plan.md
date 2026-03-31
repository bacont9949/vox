# Plan: Vox Meeting Minutes Feature

## Context

Vox v0.1 is a macOS voice input app (Option key → record → transcribe → inject text). User wants to add a "Meeting Minutes" mode: continuous recording with auto-chunking, real-time transcript accumulation, and post-meeting LLM summary output as Markdown file.

## Approach

Add Meeting Mode as a parallel state machine alongside Normal mode. Option key toggle is replaced by menu-driven start/stop. Audio is auto-chunked every 30 seconds, each chunk transcribed independently, results accumulated in order. On meeting end, full transcript is sent to LLM for structured summary, then saved as Markdown.

## Files to Modify

| File | Changes |
|------|---------|
| `app/src/meeting.rs` | **NEW** — MeetingSession, ChunkResult, Markdown generation |
| `app/src/audio.rs` | Add `drain_chunk()` — drain PCM buffer without stopping recording |
| `app/src/config.rs` | Add `MeetingConfig` (chunk_duration, output_dir, auto_summary, summary_prompt) |
| `app/src/transcribe.rs` | Add `MEETING_CHUNK_REQUEST_ID`, `send_meeting_chunk_request()` |
| `app/src/llm_refine.rs` | Add `LLM_SUMMARY_REQUEST_ID`, `send_summary_request()`, increase max_tokens |
| `app/src/main.rs` | New states, meeting flow, menu items, capsule display, chunk timer |

## Implementation Steps

### Phase 1: Foundation
1. Create `app/src/meeting.rs` — `MeetingSession` struct with chunks, timestamps, markdown generation
2. Add `MeetingConfig` to `config.rs` (chunk_duration: 30s, output_dir: ~/Documents/Vox, auto_summary: true)
3. Add `drain_chunk()` to `audio.rs` — `lock()` + `mem::take()` to drain PCM without stopping capture
4. Add `AppMode` enum (Normal/Meeting) and meeting fields to `Inner` struct

### Phase 2: Core Flow
5. Add menu items: Start Meeting / Stop Meeting (MENU_MEETING_START=30, MENU_MEETING_STOP=31)
6. Implement `start_meeting()` — begin recording, start 30s chunk timer, show capsule
7. Implement chunk timer handler — drain → encode WAV → send HTTP (sequential, one in-flight at a time)
8. Add `MEETING_CHUNK_REQUEST_ID` in `transcribe.rs`, handle response → accumulate transcript
9. Implement chunk queue (VecDeque<Vec<u8>>) for when previous chunk hasn't returned yet

### Phase 3: Summary + Output
10. Add `send_summary_request()` in `llm_refine.rs` with meeting summary system prompt (max_tokens: 4096)
11. Implement `stop_meeting()` — stop recording, drain final chunk, wait for all pending, then summarize
12. Implement `save_meeting_markdown()` — write timestamped .md file with transcript + LLM summary

### Phase 4: UI Polish
13. Capsule shows "📝 Meeting 05:32 | 11 chunks" with 1s update timer
14. Status bar icon → "📝" during meeting
15. Block normal-mode hotkey while meeting is active
16. Menu dynamically shows "Start Meeting" or "Stop Meeting" based on state

## Key Design Decisions

- **Sequential chunk sending**: One HTTP request in-flight at a time, queue remaining WAVs. Prevents ordering issues.
- **Option key during meeting**: Ignored (prevents accidental stop). Meeting stop only via menu.
- **drain_chunk() uses lock()**: OK on main thread — audio RT thread uses try_lock, contention window is O(1) for mem::take.
- **Memory**: ~1.9 MB per 30s chunk at 16kHz. Buffer drains each tick, no accumulation.
- **LLM context limit**: ~2hr meeting fits in 32k token context. Longer meetings may need truncation (documented limitation).

## Markdown Output Format

```markdown
# Meeting Minutes — 2026-04-01 14:30

**Duration:** 45m 22s
**Language:** Chinese
**Chunks:** 91

## Transcript

[00:00] 大家好，今天我们讨论...
[00:30] 第一个议题...
[01:00] ...

## Summary
(LLM generated)

## Key Points
- ...

## Decisions
- ...

## Action Items
- [ ] ...
```

## Verification

1. `cargo build -p vox` — compiles with no errors
2. `cargo clippy -p vox` — no warnings
3. Start app → MIC menu → Start Meeting → speak for 2 minutes → Stop Meeting
4. Check ~/Documents/Vox/ for .md file with timestamped transcript
5. Verify LLM summary section is present (requires MOONSHOT_API_KEY or ominix-api LLM)
6. Verify capsule shows elapsed time during recording
7. Verify normal Option-key input still works when not in meeting mode
