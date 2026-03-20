use crate::{git_ops::update_readme_ja, models::README_MESSAGE};
use std::{fs, path::PathBuf};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("norenwake-test-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn readme_is_created_when_missing() {
    let dir = temp_dir("readme-create");
    update_readme_ja(&dir, "source-repo", "my-new-repo").unwrap();
    let text = fs::read_to_string(dir.join("README.ja.md")).unwrap();
    assert!(text.contains("my-new-repo"));
    assert!(text.contains(README_MESSAGE));
}

#[test]
fn readme_intro_is_replaced_not_duplicated() {
    let dir = temp_dir("readme-replace");
    let path = dir.join("README.ja.md");
    let old = format!(
        "# my-new-repo\n\n{}\n\n# 本来のREADME\n\n本文です\n",
        README_MESSAGE
    );
    fs::write(&path, old).unwrap();

    update_readme_ja(&dir, "source-repo", "new-repo-name").unwrap();
    let text = fs::read_to_string(path).unwrap();

    assert!(text.starts_with(&format!("# new-repo-name\n\n{}\n\n", README_MESSAGE)));
    assert!(!text.contains("# my-new-repo\n\n"));
    assert!(text.contains("# 本来のREADME"));
}

#[test]
fn stacked_intros_are_collapsed_to_one() {
    let dir = temp_dir("readme-stack");
    let path = dir.join("README.ja.md");
    let old = format!(
        "# newest\n\n{}\n\n# older\n\n{}\n\n# 本来のREADME\n",
        README_MESSAGE, README_MESSAGE
    );
    fs::write(&path, old).unwrap();

    update_readme_ja(&dir, "source-repo", "final-name").unwrap();
    let text = fs::read_to_string(path).unwrap();

    assert_eq!(text.matches(README_MESSAGE).count(), 1);
    assert!(text.starts_with("# final-name\n\n"));
    assert!(text.contains("# 本来のREADME"));
}

#[test]
fn trailing_newline_is_preserved_when_present() {
    let dir = temp_dir("readme-trailing-newline-present");
    let path = dir.join("README.ja.md");
    let old = format!("# my-new-repo\n\n{}\n\n# 本来のREADME\n", README_MESSAGE);
    fs::write(&path, old).unwrap();

    update_readme_ja(&dir, "source-repo", "new-repo-name").unwrap();
    let text = fs::read_to_string(path).unwrap();

    assert!(text.ends_with('\n'));
}

#[test]
fn trailing_newline_is_preserved_when_absent() {
    let dir = temp_dir("readme-trailing-newline-absent");
    let path = dir.join("README.ja.md");
    let old = format!("# my-new-repo\n\n{}\n\n# 本来のREADME", README_MESSAGE);
    fs::write(&path, old).unwrap();

    update_readme_ja(&dir, "source-repo", "new-repo-name").unwrap();
    let text = fs::read_to_string(path).unwrap();

    assert!(!text.ends_with('\n'));
}
