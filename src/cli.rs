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

    #[arg(long, alias = "tok", default_value_t = true)]
    pub tokens: bool,

    #[arg(long, default_value_t = true)]
    pub ast: bool,

    #[arg(long)]
    pub asm: bool,

    #[arg(long, alias = "obj")]
    pub object: bool,
}
