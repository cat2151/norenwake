fn main() -> anyhow::Result<()> {
    let action = match norenwake::parse_command(std::env::args_os()) {
        Ok(action) => action,
        Err(error) => error.exit(),
    };

    match action {
        norenwake::CommandAction::RunTui => norenwake::run(),
        norenwake::CommandAction::Update => norenwake::run_self_update(),
        norenwake::CommandAction::Check => norenwake::run_check(),
    }
}
