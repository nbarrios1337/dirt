#[derive(Debug, Clone, Copy)] // TODO what other derives needed?
struct DNS_Header {
    id: u32,
    flags: u32, // TODO bitflags?
    num_questions: u32,
    num_answers: u32,
    num_authorities: u32,
    num_additionals: u32,
}

#[derive(Debug, Clone)]
struct DNS_Question {
    // will not using &[u8] be a problem? Assuming thats the equivalent for Pythons' `bytes` class
    name: String,
    class: u32,
    r#type: u8, // TODO definitely a future enum
}

fn main() {
    println!("Hello, world!");
}
