use settingspec::cli::run_cli;
use settingspec::error::SettingSpecError;

fn main() {
    if let Err(e) = run_cli() {
        match e {
            SettingSpecError::CommandFailed(code) => {
                std::process::exit(code);
            }
            err => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
