use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
struct Arguments {
    /// Requested domain name
    request: String,
}

fn main() {
    let args = Arguments::parse();

    match dirt::lookup_domain(&args.request) {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}
