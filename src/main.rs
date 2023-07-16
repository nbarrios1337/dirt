use clap::Parser;
use tracing_subscriber::prelude::*;

#[derive(Parser)]
#[command(author, version, about)]
struct Arguments {
    /// Requested domain name
    request: String,
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false);

    let filter_layer = tracing_subscriber::EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();

    let args = Arguments::parse();

    match dirt::lookup_domain(&args.request) {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}
