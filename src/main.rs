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
        let all_args: Vec<String> = parsed.into_iter().skip(1).collect();
        let (args, redirect_file) = if let Some(pos) = all_args.iter().position(|s| s == ">"){
            (all_args[..pos].to_vec(),all_args.get(pos+1).cloned())
        } else {
            (all_args, None)
        };
        match command.trim() {
            "exit" => break,
            "echo" => {
                let content = args.join(" ") + "\n";
                if let Some(file) = redirect_file {
                    std::fs::write(file, content).unwrap();
                } else {
                    println!("{}", content);
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
                    let mut cmd = Command::new(&command);
                    cmd.args(&args);
                    if let Some(file) = redirect_file {
                        let f = std::fs::File::create(file).unwrap();
                        cmd.stdout(std::process::Stdio::from(f));
                    }
                    cmd.spawn().unwrap().wait().unwrap();
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
