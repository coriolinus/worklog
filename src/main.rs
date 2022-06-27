#[macro_use]
extern crate lalrpop_util;

mod cli;

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let opts = Cli::parse();
    match opts.subcommand {
        Command::Paths {
            which: PathSubcommand::Config,
        } => {
            println!("{}", worklog::paths::config().display());
        }
        Command::Paths {
            which: PathSubcommand::Database,
        } => {
            println!("{}", worklog::paths::database().display());
        }
        other => {
            eprintln!("not yet supported:\n{:?}", other);
            panic!("unsupported subcommand");
        }
    }

    Ok(())
}
