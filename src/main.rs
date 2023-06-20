fn main() {
    match dirt::lookup_domain("www.example.com") {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}
