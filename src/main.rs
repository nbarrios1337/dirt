use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Arguments {
    /// Requested domain name
    request: String,
}

fn main() {
    let tracing_subcriber = tracing_subscriber::FmtSubscriber::new();
    if let Err(e) = tracing::subscriber::set_global_default(tracing_subcriber) {
        panic!("{e}")
    };

    let args = Arguments::parse();

    match dirt::lookup_domain(&args.request) {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}
