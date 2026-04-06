fn main() -> Result<(), Box<dyn std::error::Error>> {
    let action = match norenwake::parse_command(std::env::args_os()) {
        Ok(action) => action,
        Err(error) => error.exit(),
    };

    match action {
        norenwake::CommandAction::RunTui => norenwake::run().map_err(Into::into),
        norenwake::CommandAction::Update => norenwake::run_self_update(),
        norenwake::CommandAction::Check => norenwake::run_check(),
    }
}
