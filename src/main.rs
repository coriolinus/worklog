use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    Paths {
        #[structopt(subcommand)]
        which: PathSubcommand,
    },
}

#[derive(Debug, StructOpt)]
enum PathSubcommand {
    Database,
    Config,
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let opts = Options::from_args();
    match opts.subcommand {
        Subcommand::Paths {
            which: PathSubcommand::Config,
        } => {
            println!("{}", worklog::paths::config().display());
        }
        Subcommand::Paths {
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
