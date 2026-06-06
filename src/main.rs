#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;


const BUILTINS: &[&str] = &["exit", "echo", "type"];

enum Redirection {
    Stdout(String),
    Stderr(String)
}

struct CommandInfo {
    command: String,
    args: Vec<String>,
    redirection: Option<Redirection>
}

impl CommandInfo {
    fn new(command: String, args:Vec<String>) -> Self {
        args.iter().position(|s| s == "2>").map(|pos|{
            let redirect_file = args.get(pos+1).cloned();
            if let Some(file) = redirect_file {
                Self {
                    command:command.clone(),
                    args: args[..pos].to_vec(),
                    redirection: Some(Redirection::Stderr(file))
                }
            } else {
                Self {
                    command: command.clone(),
                    args: args[..pos].to_vec(),
                    redirection: None
                }
            }
        }).or_else(|| { 
            args.iter().position(|s| s == ">" || s == "1>").map(|pos|{
            let redirect_file = args.get(pos+1).cloned();
            if let Some(file) = redirect_file {
                Self {
                    command:command.clone(),
                    args: args[..pos].to_vec(),
                    redirection: Some(Redirection::Stdout(file))
                }
            } else {
                Self {
                    command: command.clone(),
                    args: args[..pos].to_vec(),
                    redirection: None
                }
            }
        })}).unwrap_or_else(|| Self {
            command,
            args,
            redirection: None
        })
    }
    fn redirection(&self){

    }
}

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
        let command_info = CommandInfo::new(command, all_args);

        match command_info.command.trim() {
            "exit" => break,
            "echo" => {
                let content = command_info.args.join(" ") + "\n";
                let redirect = command_info.redirection;
                match redirect {
                    Some(Redirection::Stdout(file)) => std::fs::write(file, content).unwrap(),
                    Some(Redirection::Stderr(file)) => {
                        std::fs::write(file, "").unwrap();
                        print!("{}",content);
                    },
                    _ => print!("{}", content),
                }
            },
            "type" => {
                let cmd = command_info.args.first().map(|s| s.as_str()).unwrap_or("");
                if BUILTINS.contains(&cmd) {
                    println!("{} is a shell builtin", cmd);
                } else if let Some(path) = find_in_path(cmd, &path_env) {
                    println!("{} is {}", cmd, path);
                } else {
                    println!("{}: not found", cmd);
                }
            }
            _ => {
                if find_in_path(&command_info.command, &path_env).is_some() {
                    let mut cmd = Command::new(&command_info.command);
                    cmd.args(&command_info.args);
                    match command_info.redirection {
                        Some(Redirection::Stdout(file)) => {
                            let f = std::fs::File::create(file).unwrap();
                            cmd.stdout(std::process::Stdio::from(f));
                        }
                        Some(Redirection::Stderr(file)) => {
                            let f = std::fs::File::create(file).unwrap();
                            cmd.stderr(std::process::Stdio::from(f));
                        }
                        None => {}
                    }
                    cmd.spawn().unwrap().wait().unwrap();
                    
                } else {
                    println!("{}: not found", command_info.command);
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

