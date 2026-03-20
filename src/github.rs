use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::process::Command;

use crate::models::Repo;

#[derive(Debug, Deserialize)]
struct User {
    login: String,
}

#[derive(Debug, Clone)]
pub struct ReadmeContent {
    pub file_name: &'static str,
    pub markdown: String,
}

pub fn get_github_token() -> Result<String> {
    if let Ok(v) = std::env::var("GH_TOKEN") {
        if !v.trim().is_empty() {
            return Ok(v);
        }
    }
    if let Ok(v) = std::env::var("GITHUB_TOKEN") {
        if !v.trim().is_empty() {
            return Ok(v);
        }
    }
    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .context("`gh auth token` の実行に失敗しました")?;
    if !output.status.success() {
        return Err(anyhow!("`gh auth token` が失敗しました"));
    }
    let token = String::from_utf8(output.stdout)?.trim().to_string();
    if token.is_empty() {
        return Err(anyhow!("GitHub token が空です"));
    }
    Ok(token)
}

fn client(_token: &str) -> Result<Client> {
    Client::builder()
        .user_agent("norenwake")
        .build()
        .context("reqwest client の作成に失敗しました")
}

pub fn fetch_repo_readme_ja(token: &str, full_name: &str) -> Result<ReadmeContent> {
    let client = client(token)?;
    let primary = format!(
        "https://api.github.com/repos/{}/contents/README.ja.md",
        full_name
    );
    let fallback = format!(
        "https://api.github.com/repos/{}/contents/README.md",
        full_name
    );
    let first = client
        .get(&primary)
        .bearer_auth(token)
        .header("Accept", "application/vnd.github.raw+json")
        .send()
        .with_context(|| format!("README.ja.md request に失敗しました: {}", full_name))?;
    if first.status().is_success() {
        let markdown = first
            .text()
            .with_context(|| format!("README.ja.md text の取得に失敗しました: {}", full_name))?;
        return Ok(ReadmeContent {
            file_name: "README.ja.md",
            markdown,
        });
    }

    let markdown = client
        .get(&fallback)
        .bearer_auth(token)
        .header("Accept", "application/vnd.github.raw+json")
        .send()
        .with_context(|| format!("README.md request に失敗しました: {}", full_name))?
        .error_for_status()
        .with_context(|| format!("README.md status エラーです: {}", full_name))?
        .text()
        .with_context(|| format!("README.md text の取得に失敗しました: {}", full_name))?;
    Ok(ReadmeContent {
        file_name: "README.md",
        markdown,
    })
}

pub fn fetch_authenticated_login(token: &str) -> Result<String> {
    let client = client(token)?;
    let user: User = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .send()
        .context("/user request に失敗しました")?
        .error_for_status()
        .context("/user status エラーです")?
        .json()
        .context("/user json parse に失敗しました")?;
    Ok(user.login)
}

pub fn repo_is_selectable(repo: &Repo) -> bool {
    !repo.private && !repo.fork && !repo.archived
}

pub fn sort_repos(mut repos: Vec<Repo>) -> Vec<Repo> {
    repos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    repos
}

pub fn fetch_repos(token: &str) -> Result<Vec<Repo>> {
    let client = client(token)?;
    let mut all = Vec::new();
    let mut page = 1;
    loop {
        let url = format!("https://api.github.com/user/repos?per_page=100&page={}&sort=updated&direction=desc&affiliation=owner", page);
        let items: Vec<Repo> = client
            .get(&url)
            .bearer_auth(token)
            .send()
            .with_context(|| format!("repo request に失敗しました: page={}", page))?
            .error_for_status()
            .context("repo status エラーです")?
            .json()
            .context("repo json parse に失敗しました")?;
        if items.is_empty() {
            break;
        }
        all.extend(items);
        page += 1;
    }
    Ok(sort_repos(
        all.into_iter().filter(repo_is_selectable).collect(),
    ))
}

pub fn check_repo_name_available(token: &str, login: &str, repo_name: &str) -> Result<bool> {
    let client = client(token)?;
    let url = format!("https://api.github.com/repos/{}/{}", login, repo_name);
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .context("repo availability request に失敗しました")?;
    Ok(res.status().as_u16() == 404)
}
