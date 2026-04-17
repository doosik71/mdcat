use std::path::PathBuf;
use std::process::ExitCode;
use std::io::IsTerminal;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "mdcat", version, about = "Terminal Markdown renderer")]
struct Args {
    #[arg(long = "theme", value_enum, default_value_t = ThemeChoice::Dark)]
    theme: ThemeChoice,

    #[arg(long = "no-color")]
    no_color: bool,

    path: PathBuf,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ThemeChoice {
    Dark,
    Light,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let color_enabled = !args.no_color && std::io::stdout().is_terminal();
    let theme = match args.theme {
        ThemeChoice::Dark => mdcat::Theme::Dark,
        ThemeChoice::Light => mdcat::Theme::Light,
    };

    match mdcat::render_file_with_theme(&args.path, color_enabled, theme) {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("mdcat: {err}");
            ExitCode::from(1)
        }
    }
}
