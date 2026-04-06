use std::path::{Path, PathBuf};
use std::time::Instant;

/// A single chunk of transcribed audio.
pub struct ChunkResult {
    pub index: u32,
    pub text: String,
    pub timestamp_offset_secs: f64,
}

/// State for an active meeting session.
pub struct MeetingSession {
    pub start_time: Instant,
    pub start_datetime: String,
    pub chunks: Vec<ChunkResult>,
    pub pending_chunks: u32,
    pub next_chunk_index: u32,
    pub output_dir: PathBuf,
    file_handle: Option<std::fs::File>,
}

impl MeetingSession {
    pub fn new(output_dir: &Path, language: &str) -> Self {
        let now = chrono_now();
        let start_datetime = now.clone();
        let session_dir = output_dir.join(format!("meeting_{}", now));
        let _ = std::fs::create_dir_all(&session_dir);

        // Create transcript.md with header immediately
        let transcript_path = session_dir.join("transcript.md");
        let header = format!(
            "# Meeting Transcript — {}\n\n**Language:** {}\n\n---\n\n",
            format_display_time(&now),
            language,
        );
        let file_handle = std::fs::File::create(&transcript_path)
            .ok()
            .map(|mut f| {
                use std::io::Write;
                let _ = f.write_all(header.as_bytes());
                f
            });

        Self {
            start_time: Instant::now(),
            start_datetime,
            chunks: Vec::new(),
            pending_chunks: 0,
            next_chunk_index: 0,
            output_dir: session_dir,
            file_handle,
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    pub fn elapsed_display(&self) -> String {
        let secs = self.elapsed_secs() as u64;
        let m = secs / 60;
        let s = secs % 60;
        format!("{:02}:{:02}", m, s)
    }

    /// Add a chunk result and immediately append to transcript.md.
    pub fn add_chunk_result(&mut self, text: &str) {
        let offset = self.chunks.len() as f64 * 30.0; // approximate
        let index = self.chunks.len() as u32;

        let chunk = ChunkResult {
            index,
            text: text.to_string(),
            timestamp_offset_secs: offset,
        };

        // Append to file immediately
        if let Some(ref mut f) = self.file_handle {
            use std::io::Write;
            let ts = format_timestamp(offset);
            let _ = writeln!(f, "[{}] {}\n", ts, text);
            let _ = f.flush();
        }

        self.chunks.push(chunk);
    }

    /// Get the full transcript text (for LLM summary).
    pub fn full_transcript(&self) -> String {
        self.chunks
            .iter()
            .map(|c| {
                let ts = format_timestamp(c.timestamp_offset_secs);
                format!("[{}] {}", ts, c.text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get the latest chunk text (for capsule display).
    pub fn latest_text(&self) -> &str {
        self.chunks
            .last()
            .map(|c| c.text.as_str())
            .unwrap_or("")
    }

    /// Finalize transcript.md with duration and chunk count.
    pub fn finalize_transcript(&mut self) {
        let elapsed = self.elapsed_display();
        let chunk_count = self.chunks.len();
        if let Some(ref mut f) = self.file_handle {
            use std::io::Write;
            let _ = writeln!(
                f,
                "\n---\n\n**Duration:** {}\n**Chunks:** {}\n",
                elapsed,
                chunk_count
            );
            let _ = f.flush();
        }
    }

    /// Save summary.md with LLM-generated content.
    pub fn save_summary(&self, summary: &str) -> Result<(), String> {
        let path = self.output_dir.join("summary.md");
        let content = format!(
            "# Meeting Summary — {}\n\n**Duration:** {}\n**Chunks:** {}\n\n---\n\n{}\n",
            format_display_time(&self.start_datetime),
            self.elapsed_display(),
            self.chunks.len(),
            summary,
        );
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Get the session output directory path.
    pub fn output_path(&self) -> &Path {
        &self.output_dir
    }
}

fn format_timestamp(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{:02}:{:02}", m, s)
}

fn format_display_time(datetime_str: &str) -> String {
    // Convert "2026-04-01_143022" to "2026-04-01 14:30"
    if datetime_str.len() >= 15 {
        let date = &datetime_str[..10];
        let h = &datetime_str[11..13];
        let m = &datetime_str[13..15];
        format!("{} {}:{}", date, h, m)
    } else {
        datetime_str.to_string()
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert unix timestamp to YYYY-MM-DD_HHMMSS
    // Simple calculation (not timezone-aware, but good enough)
    let secs_per_day = 86400u64;
    let days = now / secs_per_day;
    let day_secs = now % secs_per_day;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;

    // Days since epoch to Y-M-D (simplified, no leap second handling)
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}_{:02}{:02}{:02}", year, month, day, h, m, s)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified date calculation from days since epoch (1970-01-01)
    let mut y = 1970u64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months_days: [u64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for &md in &months_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    (y, m, remaining + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
