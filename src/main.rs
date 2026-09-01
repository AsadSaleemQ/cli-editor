fn main() {
    match codex_cli_editor::run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("codex-cli-editor: {error}");
            std::process::exit(error.exit_code());
        }
    }
}
