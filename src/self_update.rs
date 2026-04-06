use cat_self_update_lib::{check_remote_commit, self_update};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const REPO_OWNER: &str = "cat2151";
const REPO_NAME: &str = "norenwake";
const MAIN_BRANCH: &str = "main";
const BIN_NAME: &str = "norenwake";
const LOCAL_HASH: &str = env!("GIT_COMMIT_HASH");

pub fn run_self_update() -> Result<()> {
    println!("セルフアップデートを開始します...");
    self_update(REPO_OWNER, REPO_NAME, &[BIN_NAME])?;
    Ok(())
}

pub fn run_check() -> Result<()> {
    let result = check_remote_commit(REPO_OWNER, REPO_NAME, MAIN_BRANCH, LOCAL_HASH)?;
    println!("{result}");
    Ok(())
}
