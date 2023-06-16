fn main() {
    match dirc::lookup_domain("www.example.com") {
        Ok(ip) => println!("{ip}"),
        Err(e) => eprintln!("{e}"),
    }
}
