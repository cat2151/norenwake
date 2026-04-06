use crate::{parse_command, CommandAction};
use clap::error::ErrorKind;

#[test]
fn no_subcommand_runs_tui() {
    let action = parse_command(["norenwake"]).unwrap();
    assert_eq!(action, CommandAction::RunTui);
}

#[test]
fn update_and_check_subcommands_are_detected() {
    assert_eq!(
        parse_command(["norenwake", "update"]).unwrap(),
        CommandAction::Update
    );
    assert_eq!(
        parse_command(["norenwake", "check"]).unwrap(),
        CommandAction::Check
    );
}

#[test]
fn help_lists_supported_subcommands() {
    let error = parse_command(["norenwake", "--help"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::DisplayHelp);

    let rendered = error.to_string();
    assert!(rendered.contains("update"));
    assert!(rendered.contains("check"));
}

#[test]
fn unknown_subcommand_is_rejected() {
    let error = parse_command(["norenwake", "upgrade"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidSubcommand);
}
