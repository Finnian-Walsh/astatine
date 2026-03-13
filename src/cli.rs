use clap::Parser;

#[derive(Parser)]
#[command(
    name = "astatine compiler",
    about = "The official compiler for the astatine programming language"
)]
pub struct Cli {
    pub file_name: String,

    #[arg(short)]
    pub output: Option<String>,
}
