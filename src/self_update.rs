use anyhow::Result;
use std::process::Command;

const GIT_URL: &str = "https://github.com/cat2151/norenwake";
#[cfg(target_os = "windows")]
const MAX_TEMP_FILE_ATTEMPTS: u32 = 100;

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
        use std::io::ErrorKind;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        let pid = std::process::id();
        let timestamp_str = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .map(|millis| millis.to_string())
            .unwrap_or_else(|_| "pre_epoch".to_string());
        let temp_dir = std::env::temp_dir();
        let mut bat_path = None;
        for attempt in 0..MAX_TEMP_FILE_ATTEMPTS {
            let candidate = temp_dir.join(format!(
                "norenwake_update_{pid}_{timestamp_str}_{attempt}.bat"
            ));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
            {
                Ok(mut file) => {
                    file.write_all(update_bat_content().as_bytes())?;
                    bat_path = Some(candidate);
                    break;
                }
                Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err.into()),
            }
        }
        let bat_path =
            bat_path.ok_or_else(|| anyhow::anyhow!("failed to allocate a unique temp bat path"))?;

        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&bat_path)
            .spawn()
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to launch update script {} via cmd.exe: {err}",
                    bat_path.display()
                )
            })?;

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
            anyhow::bail!("cargo install failed with status: {status}");
        }
        Ok(false)
    }
}
