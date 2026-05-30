mod command {
    pub mod replay;
}

mod protocol;

use core::str;

use clap::Parser;
use jane_eyre::eyre;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(clap::Parser, Debug)]
enum Args {
    Replay(crate::command::replay::Replay),
}

fn main() -> eyre::Result<()> {
    jane_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive("trace".parse()?)
                .from_env()?,
        )
        .init();
    let args = Args::parse();

    match args {
        Args::Replay(args) => crate::command::replay::main(args),
    }
}
