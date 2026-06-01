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
            "echo" => println!("{}", args.join(" ")),
            "type" => {
                let arg_second = args.first().map(|s| s.as_str()).unwrap_or("");
                if BUILTINS.contains(&arg_second) {
                    println!("{} is a shell builtin", arg_second);
                } else {
                    let found = path_env.split(":").any(|path| {
                        let file_path = format!("{}/{}", path, arg_second);
                        if std::path::Path::new(&file_path).exists() {
                            let meta = std::fs::metadata(&file_path).unwrap();
                            if meta.permissions().mode() & 0o111 != 0 {
                                println!("{} is {}", arg_second, &file_path);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if !found {
                        println!("{}: not found", arg_second);
                    }
                }
            }
            _ => {
                let found: bool = path_env.split(":").any(|path| {
                    let file_path: String = format!("{}/{}", path, command);
                    if std::path::Path::new(&file_path).exists() {
                        let meta = std::fs::metadata(&file_path).unwrap();
                        if meta.permissions().mode() & 0o111 != 0 {
                            let mut child = Command::new(&command).args(&args).spawn().unwrap();
                            child.wait().unwrap();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                if !found {
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
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
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
