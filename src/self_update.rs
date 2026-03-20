use anyhow::{bail, Result};
use std::process::Command;

const GIT_URL: &str = "https://github.com/cat2151/norenwake";

pub fn should_handle_update_subcommand(args: &[String]) -> bool {
    args.get(1).map(String::as_str) == Some("update")
}

fn install_cmd() -> String {
    format!("cargo install --force --git {GIT_URL}")
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn update_bat_content() -> String {
    format!(
        "@echo off\r\ntimeout /t 3 /nobreak >nul\r\n{cmd}\r\ndel \"%~f0\"\r\n",
        cmd = install_cmd()
    )
}

pub fn run_self_update() -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        let pid = std::process::id();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let bat_path = std::env::temp_dir().join(format!("norenwake_update_{pid}_{ts}.bat"));
        {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&bat_path)?;
            file.write_all(update_bat_content().as_bytes())?;
        }

        let bat_str = bat_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("temp bat path is not valid UTF-8"))?;
        Command::new("cmd")
            .args(["/C", "start", "", bat_str])
            .spawn()?;

        println!("Launching update script: {}", bat_path.display());
        println!("The application will now exit so the file lock is released.");
        return Ok(true);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = install_cmd();
        println!("Running: {cmd}");
        let status = Command::new("cargo")
            .args(["install", "--force", "--git", GIT_URL])
            .status()?;
        if !status.success() {
            bail!("cargo install failed with status: {status}");
        }
        Ok(false)
    }
}
