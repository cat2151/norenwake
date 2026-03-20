use anyhow::{anyhow, Context, Result};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{logging::append_log_line, models::README_MESSAGE, util::now_string};

pub fn run_cmd_logged(log_path: &Path, cwd: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    append_log_line(
        log_path,
        &format!(
            "[{}] $ (cwd={}) {} {}",
            now_string(),
            cwd.display(),
            cmd,
            args.join(" ")
        ),
    )?;
    let output = Command::new(cmd)
        .current_dir(cwd)
        .args(args)
        .output()
        .with_context(|| format!("command の実行に失敗しました: {} {}", cmd, args.join(" ")))?;
    if !output.stdout.is_empty() {
        append_log_line(
            log_path,
            &format!(
                "[{}] stdout: {}",
                now_string(),
                String::from_utf8_lossy(&output.stdout).trim_end()
            ),
        )?;
    }
    if !output.stderr.is_empty() {
        append_log_line(
            log_path,
            &format!(
                "[{}] stderr: {}",
                now_string(),
                String::from_utf8_lossy(&output.stderr).trim_end()
            ),
        )?;
    }
    if !output.status.success() {
        let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_text = output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let mut detail = format!(
            "command failed (exit={}): {} {}",
            exit_text,
            cmd,
            args.join(" ")
        );
        if let Some(first_stderr) = stderr_text.lines().find(|line| !line.trim().is_empty()) {
            let snippet = if first_stderr.len() > 180 {
                format!("{}...", &first_stderr[..180])
            } else {
                first_stderr.to_string()
            };
            detail.push_str(&format!(" / stderr: {}", snippet));
        }
        if cmd == "git"
            && args.len() >= 2
            && args[0] == "push"
            && (stderr_text.contains("Invalid username or token")
                || stderr_text.contains("Authentication failed")
                || stderr_text.contains("credential-manager-core"))
        {
            detail.push_str(
                " / hint: origin の push URL を SSH に設定するか、HTTPS 認証設定を確認してください",
            );
        }
        if cmd == "git"
            && args.len() >= 2
            && args[0] == "push"
            && stderr_text.contains("without `workflow` scope")
        {
            detail.push_str(
                " / hint: SSH push を使うか、workflow scope 付きの HTTPS 認証を使ってください",
            );
        }
        return Err(anyhow!(detail));
    }
    append_log_line(log_path, &format!("[{}] success", now_string()))?;
    Ok(())
}

pub fn build_target_origin_url(login: &str, repo_name: &str) -> String {
    format!("https://github.com/{}/{}.git", login, repo_name)
}

pub fn build_target_origin_push_url(login: &str, repo_name: &str) -> String {
    format!("git@github.com:{}/{}.git", login, repo_name)
}

pub fn ensure_safe_remotes(
    log_path: &Path,
    repo_dir: &Path,
    _source_clone_url: &str,
    target_origin_url: &str,
    target_push_url: &str,
) -> Result<()> {
    if git_remote_url(repo_dir, "upstream")
        .ok()
        .flatten()
        .is_some()
    {
        let _ = run_cmd_logged(log_path, repo_dir, "git", &["remote", "remove", "upstream"]);
    }
    let current_origin = git_remote_url(repo_dir, "origin").ok().flatten();
    match current_origin {
        Some(ref url) if url == target_origin_url => {}
        Some(_) => {
            let _ = run_cmd_logged(
                log_path,
                repo_dir,
                "git",
                &["remote", "set-url", "origin", target_origin_url],
            );
        }
        None => {
            let _ = run_cmd_logged(
                log_path,
                repo_dir,
                "git",
                &["remote", "add", "origin", target_origin_url],
            );
        }
    }
    let _ = run_cmd_logged(
        log_path,
        repo_dir,
        "git",
        &["remote", "set-url", "--push", "origin", target_push_url],
    );
    Ok(())
}

pub fn git_remote_url(repo_dir: &Path, remote: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(["remote", "get-url", remote])
        .output()
        .context("`git remote get-url` の実行に失敗しました")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8(output.stdout)?.trim().to_string()))
}

pub fn git_remote_push_url(repo_dir: &Path, remote: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(["remote", "get-url", "--push", remote])
        .output()
        .context("`git remote get-url --push` の実行に失敗しました")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8(output.stdout)?.trim().to_string()))
}

pub fn rename_preview_dir(current_dir: &Path, repo_name: &str) -> Result<PathBuf> {
    let parent = current_dir
        .parent()
        .ok_or_else(|| anyhow!("parent dir がありません"))?;
    let target = parent.join(repo_name);
    if current_dir == target {
        return Ok(target);
    }
    if target.exists() {
        fs::remove_dir_all(&target).with_context(|| {
            format!("既存 target dir の削除に失敗しました: {}", target.display())
        })?;
    }
    fs::rename(current_dir, &target).with_context(|| {
        format!(
            "dir rename に失敗しました: {} -> {}",
            current_dir.display(),
            target.display()
        )
    })?;
    Ok(target)
}

pub fn update_readme_ja(repo_dir: &Path, _source_name: &str, new_repo_name: &str) -> Result<()> {
    let path = repo_dir.join("README.ja.md");
    let content = if path.exists() {
        let old = fs::read_to_string(&path).unwrap_or_default();
        let had_trailing_newline = old.ends_with('\n');
        let tail = strip_existing_norenwake_intro(&old);
        let tail = tail.trim_start_matches(['\r', '\n']);
        let mut updated = if tail.is_empty() {
            format!("# {}\n\n{}", new_repo_name, README_MESSAGE)
        } else {
            format!("# {}\n\n{}\n\n{}", new_repo_name, README_MESSAGE, tail)
        };
        if had_trailing_newline {
            updated.push('\n');
        }
        updated
    } else {
        format!("# {}\n\n{}\n", new_repo_name, README_MESSAGE)
    };
    fs::write(&path, content)
        .with_context(|| format!("README.ja.md の更新に失敗しました: {}", path.display()))?;
    Ok(())
}

pub fn render_delta_for_file(repo_dir: &Path, file: &str) -> Result<(String, Vec<String>)> {
    let diff_output = Command::new("git")
        .current_dir(repo_dir)
        .args(["diff", "--", file])
        .output()
        .context("`git diff` の実行に失敗しました")?;
    let diff_text = String::from_utf8(diff_output.stdout)?;
    if diff_text.trim().is_empty() {
        return Ok((String::new(), vec!["(差分はありません)".to_string()]));
    }

    let mut delta = Command::new("delta")
        .current_dir(repo_dir)
        .args([
            "--no-gitconfig",
            "--paging=never",
            "--width=96",
            "--file-style=omit",
            "--hunk-header-style=omit",
            "--keep-plus-minus-markers",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("`delta` の起動に失敗しました")?;

    if let Some(stdin) = delta.stdin.as_mut() {
        stdin
            .write_all(diff_text.as_bytes())
            .context("`delta` への入力書き込みに失敗しました")?;
    }

    let rendered = delta
        .wait_with_output()
        .context("`delta` の実行に失敗しました")?;
    if !rendered.status.success() {
        return Err(anyhow!("`delta` の実行結果が失敗しました"));
    }

    let ansi_text = String::from_utf8(rendered.stdout)?;
    let plain_text = strip_ansi_sequences(&ansi_text);
    let mut filtered_ansi_lines = Vec::new();
    let mut filtered_plain_lines = Vec::new();
    for (ansi_line, plain_line) in ansi_text.lines().zip(plain_text.lines()) {
        if is_delta_noise_line(plain_line) {
            continue;
        }
        filtered_ansi_lines.push(ansi_line.to_string());
        filtered_plain_lines.push(plain_line.to_string());
    }
    let filtered_ansi = filtered_ansi_lines.join("\n");
    Ok((
        filtered_ansi,
        if filtered_plain_lines.is_empty() {
            vec!["(delta の出力は空です)".to_string()]
        } else {
            filtered_plain_lines
        },
    ))
}

pub fn collect_files(root: &Path, limit: usize) -> Result<Vec<String>> {
    let mut acc = Vec::new();
    collect_impl(root, root, "", true, limit, &mut acc)?;
    if acc.is_empty() {
        acc.push("(empty)".to_string());
    }
    Ok(acc)
}

fn collect_impl(
    root: &Path,
    dir: &Path,
    prefix: &str,
    is_root: bool,
    limit: usize,
    acc: &mut Vec<String>,
) -> Result<()> {
    if acc.len() >= limit {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)?.collect::<std::result::Result<Vec<_>, _>>()?;
    entries.retain(|entry| {
        entry
            .file_name()
            .to_str()
            .map(|name| name != ".git")
            .unwrap_or(true)
    });
    entries.sort_by_key(|entry| {
        let path = entry.path();
        (!path.is_dir(), entry.file_name())
    });
    for (index, entry) in entries.iter().enumerate() {
        if acc.len() >= limit {
            break;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| {
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string()
            });
        let is_last = index + 1 == entries.len();
        let branch = if is_root {
            ""
        } else if is_last {
            "`-- "
        } else {
            "|-- "
        };
        let suffix = if path.is_dir() { "/" } else { "" };
        acc.push(format!("{}{}{}{}", prefix, branch, name, suffix));
        if path.is_dir() {
            let child_prefix = if is_root {
                String::new()
            } else if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}|   ", prefix)
            };
            collect_impl(root, &path, &child_prefix, false, limit, acc)?;
        }
    }
    Ok(())
}

pub fn git_status_lines(repo_dir: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo_dir)
        .args(["status", "--short"])
        .output()
        .context("`git status` の実行に失敗しました")?;
    let text = String::from_utf8(output.stdout)?;
    let lines: Vec<String> = text.lines().map(ToString::to_string).collect();
    Ok(if lines.is_empty() {
        vec!["(clean)".to_string()]
    } else {
        lines
    })
}

fn strip_ansi_sequences(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn strip_existing_norenwake_intro(input: &str) -> String {
    let mut rest = input.trim_start_matches(['\r', '\n']).to_string();
    loop {
        let lines: Vec<&str> = rest.lines().collect();
        if lines.is_empty() || !lines[0].starts_with("# ") {
            break;
        }

        let mut i = 1usize;
        while i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
        if i >= lines.len() || lines[i].trim() != README_MESSAGE {
            break;
        }
        i += 1;

        while i < lines.len() && lines[i].trim().is_empty() {
            i += 1;
        }
        if i < lines.len() && lines[i].trim().starts_with("元repo: `") {
            i += 1;
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
        }

        rest = lines[i..].join("\n");
        rest = rest.trim_start_matches(['\r', '\n']).to_string();
    }
    rest
}

fn is_delta_noise_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.starts_with("diff --git ")
        || trimmed.starts_with("index ")
        || trimmed.starts_with("--- ")
        || trimmed.starts_with("+++ ")
        || trimmed.starts_with("@@")
    {
        return true;
    }
    if trimmed.chars().all(|ch| {
        matches!(
            ch,
            '─' | '━' | '│' | '┤' | '├' | '┬' | '┴' | '┼' | '-' | ' '
        )
    }) {
        return true;
    }
    if let Some(prefix) = trimmed.strip_suffix(':') {
        if !prefix.is_empty() && prefix.chars().all(|ch| ch.is_ascii_digit()) {
            return true;
        }
    }
    false
}

pub fn build_remote_safety_lines(
    source_clone_url: &str,
    origin_url: Option<&str>,
    origin_push_url: Option<&str>,
    upstream_url: Option<&str>,
    expected_origin: &str,
    expected_push: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    let origin = origin_url.unwrap_or("(none)");
    let origin_push = origin_push_url.unwrap_or("(none)");
    let upstream = upstream_url.unwrap_or("(none)");
    out.push(format!("origin:   {}", origin));
    out.push(format!("pushurl:  {}", origin_push));
    out.push(format!("upstream: {}", upstream));
    if origin == source_clone_url {
        out.push("NG: origin が暖簾分け元を向いています".to_string());
    } else if origin == expected_origin {
        out.push("OK: origin は新しい repo を向いています".to_string());
    } else {
        out.push("NG: origin が想定 push 先を向いていません".to_string());
    }
    if origin_push == expected_push {
        out.push("OK: push URL は SSH を向いています".to_string());
    } else if origin_push == "(none)" {
        out.push("NG: push URL が未設定です".to_string());
    } else {
        out.push("NG: push URL が想定の SSH 宛先を向いていません".to_string());
    }
    if upstream == "(none)" {
        out.push("OK: upstream は未設定です".to_string());
    } else if upstream == source_clone_url {
        out.push("NG: upstream が暖簾分け元を向いています".to_string());
    } else {
        out.push("NG: upstream が想定外の値です".to_string());
    }
    out
}
