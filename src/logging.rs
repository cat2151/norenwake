use anyhow::{Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn ensure_log_file(path: &Path) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, b"")?;
    }
    Ok(())
}

pub fn append_log_line(path: &Path, line: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("log open 失敗: {}", path.display()))?;
    writeln!(file, "{}", line)?;
    file.flush()?;
    Ok(())
}

pub fn tail_log_lines(path: &Path, max_lines: usize) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    if lines.len() > max_lines {
        let start = lines.len() - max_lines;
        lines = lines[start..].to_vec();
    }
    Ok(lines)
}
