use std::process::Command;

pub fn get_email_list() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("(himalaya --output json list -s 0)")
        .output()
        .expect("failed to execute himalaya");
    String::from_utf8(output.stdout).unwrap()
}

/// Process himalaya command and return in JSON format string.
pub fn himalaya_command(command: Vec<String>) -> String {
    let mut iter = command.iter();
    let mut line = String::from("himalaya --output json");
    let mut options = "";

    match iter.next().unwrap().to_uppercase().as_str() {
        "SEARCH" => {
            if let Some(first) = command.get(1) {
                let keywords = [
                    "all", "answered", "before", "body", "deleted", "from", "header", "new", "not",
                    "or", "recent", "seen", "subject", "text", "to",
                ];
                if !keywords.contains(&first.to_lowercase().as_str()) {
                    line = format!("{} search subject", line);
                }
                // get all emails satified the filter
                options = "-s 0"
            }
        }
        "READ" => line = format!("{} read", line),
        _ => {}
    }

    for arg in iter {
        line = format!("{} {}", line, arg);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", line, options))
        .output()
        .expect("failed to execute himalaya");

    String::from_utf8(output.stdout).unwrap()
}
