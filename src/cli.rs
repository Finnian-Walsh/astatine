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

    #[cfg_attr(debug_assertions, arg(default_value_t = true))]
    #[arg(long, alias = "tok")]
    pub tokens: bool,

    #[cfg_attr(debug_assertions, arg(default_value_t = true))]
    #[arg(long)]
    pub ast: bool,

    #[cfg_attr(debug_assertions, arg(default_value_t = true))]
    #[arg(long)]
    pub asm: bool,

    #[cfg_attr(debug_assertions, arg(default_value_t = true))]
    #[arg(long, alias = "obj")]
    pub object: bool,
}
