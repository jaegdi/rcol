use regex::Regex;

fn strip_ansi(s: &str) -> String {
    let ansi_regex = Regex::new(r"(\x1b\[[0-9;?]*[a-zA-Z])|(\x1b\].*?(\x07|\x1b\\))").unwrap();
    ansi_regex.replace_all(s, "").to_string()
}

fn main() {
    // Construct the string: ESC ] 8 ; ; URL ESC \ Text ESC ] 8 ; ; ESC \
    // \x1b]8;;file:///home/dirk/devel/rust/kpasscli/Cargo.lock\x1b\Cargo.lock\x1b]8;;\x1b\
    let s =
        "\x1b]8;;file:///home/dirk/devel/rust/kpasscli/Cargo.lock\x1b\\Cargo.lock\x1b]8;;\x1b\\";

    println!("Original: {:?}", s);
    let stripped = strip_ansi(s);
    println!("Stripped: {:?}", stripped);

    if stripped == "Cargo.lock" {
        println!("SUCCESS");
    } else {
        println!("FAILURE");
    }
}
