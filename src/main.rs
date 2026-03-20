fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if norenwake::should_handle_update_subcommand(&args) {
        let should_exit = norenwake::run_self_update()?;
        if should_exit {
            std::process::exit(0);
        }
        return Ok(());
    }

    norenwake::run()
}
