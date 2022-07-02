use worklog::action::Action;

mod cli;
use crate::cli::Cli;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<_> = std::env::args().skip(1).collect();
    let args = args.join(" ");
    let action: Action = Cli::parse(&args)?.into();
    action.execute()?;

    Ok(())
}
