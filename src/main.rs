fn main() {
    match cli_editor::run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("cli-editor: {error}");
            std::process::exit(error.exit_code());
        }
    }
}
