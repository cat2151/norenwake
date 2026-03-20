use crate::{
    github::{repo_is_selectable, sort_repos},
    models::Repo,
};

fn repo(name: &str, updated_at: &str, private: bool, fork: bool, archived: bool) -> Repo {
    Repo {
        name: name.to_string(),
        full_name: format!("me/{}", name),
        clone_url: format!("https://github.com/me/{}.git", name),
        default_branch: "main".to_string(),
        private,
        fork,
        archived,
        updated_at: updated_at.to_string(),
        description: None,
    }
}

#[test]
fn repo_filter_matches_product_rules() {
    assert!(repo_is_selectable(&repo(
        "a",
        "2026-01-01T00:00:00Z",
        false,
        false,
        false
    )));
    assert!(!repo_is_selectable(&repo(
        "b",
        "2026-01-01T00:00:00Z",
        true,
        false,
        false
    )));
    assert!(!repo_is_selectable(&repo(
        "c",
        "2026-01-01T00:00:00Z",
        false,
        true,
        false
    )));
    assert!(!repo_is_selectable(&repo(
        "d",
        "2026-01-01T00:00:00Z",
        false,
        false,
        true
    )));
}

#[test]
fn repo_sort_is_descending_by_updated_at() {
    let repos = vec![
        repo("old", "2026-01-01T00:00:00Z", false, false, false),
        repo("new", "2026-02-01T00:00:00Z", false, false, false),
    ];
    let sorted = sort_repos(repos);
    assert_eq!(sorted[0].name, "new");
    assert_eq!(sorted[1].name, "old");
}
