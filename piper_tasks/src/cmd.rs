use std::process::Command;
use std::str;
use std::env;
use std::collections::HashMap;

/// Runs a given command using the default system shell.
pub fn run(args: &HashMap<String, String>) -> (String, String) {
    let cmd = args.get("cmd").unwrap().to_owned();

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("failed to execute process");
    let stdout = str::from_utf8(&output.stdout).unwrap().to_owned();
    let stderr = str::from_utf8(&output.stderr).unwrap().to_owned();
    
    (stdout, stderr)
}
