use crate::{should_handle_update_subcommand, update_bat_content};

#[test]
fn update_subcommand_is_detected_only_for_exact_first_arg() {
    assert!(should_handle_update_subcommand(&[
        "norenwake".to_string(),
        "update".to_string(),
    ]));
    assert!(!should_handle_update_subcommand(&["norenwake".to_string()]));
    assert!(!should_handle_update_subcommand(&[
        "norenwake".to_string(),
        "upgrade".to_string(),
    ]));
    assert!(!should_handle_update_subcommand(&[
        "norenwake".to_string(),
        "--help".to_string(),
        "update".to_string(),
    ]));
}

#[test]
fn update_bat_content_waits_installs_and_deletes_itself() {
    let bat = update_bat_content();

    assert!(bat.starts_with("@echo off\r\n"));
    assert!(bat.contains("timeout /t 3 /nobreak >nul\r\n"));
    assert!(bat.contains("cargo install --force --git https://github.com/cat2151/norenwake\r\n"));
    assert!(bat.ends_with("del \"%~f0\"\r\n"));
}
