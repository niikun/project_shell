#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const BUILTINS: &[&str] = &["exit", "echo", "type"];

fn main() {
    let path_env = std::env::var("PATH").unwrap_or(String::from(""));
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut inputs = String::new();

        io::stdin().read_line(&mut inputs).unwrap();
        let parsed = parse_args(&inputs);
        let command = parsed.first().cloned().unwrap_or_default();
        let args: Vec<String> = parsed.into_iter().skip(1).collect();
        match command.trim() {
            "exit" => break,
            "echo" => {
                if let Some(pos) = args.iter().position(|s| s == ">") {
                    let output_file = &args[pos + 1];
                    let content = args[..pos].join(" ");
                    std::fs::write(output_file, content).unwrap();
                } else {
                    println!("{}", args.join(" "));
                }
            },
            "type" => {
                let cmd = args.first().map(|s| s.as_str()).unwrap_or("");
                if BUILTINS.contains(&cmd) {
                    println!("{} is a shell builtin", cmd);
                } else if let Some(path) = find_in_path(cmd, &path_env) {
                    println!("{} is {}", cmd, path);
                } else {
                    println!("{}: not found", cmd);
                }
            }
            _ => {
                if find_in_path(&command, &path_env).is_some() {
                    Command::new(&command).args(&args).spawn().unwrap().wait().unwrap();
                } else {
                    println!("{}: not found", command);
                }
            }
        }
    }
}

fn parse_args(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote: bool = false;
    let mut is_backslashed:bool = false;

    for ch in input.chars() {
        if !is_backslashed{
            match ch {
                '\\' if !in_single_quote => is_backslashed = !is_backslashed,
                '\\' if in_single_quote => current.push(ch),
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '\"' if in_single_quote => current.push(ch),
                '\"' => in_double_quote = !in_double_quote,
                ' ' | '\t' if !in_single_quote && !in_double_quote => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                '\n' => break,
                _ => current.push(ch),
            }
        } else {
            current.push(ch);
            is_backslashed = !is_backslashed;
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn find_in_path(command: &str, path_env: &str) -> Option<String> {
    path_env.split(":").find_map(|dir| {
        let path = format!("{}/{}",dir, command);
        let meta = std::fs::metadata(&path).ok()?;
        if meta.permissions().mode() & 0o111 != 0 {
            Some(path)
        } else {
            None
        }
    })
}
