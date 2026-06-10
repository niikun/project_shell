use rustyline::Context;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, Editor, Result};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
#[allow(unused_imports)]
use std::error::Error;
use std::fs;
use std::fs::{OpenOptions, write};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Helper, Hinter, Highlighter, Validator)]
struct MyHelper {
    commands: Vec<String>,
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        context: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let current_word = line.split_whitespace().last().unwrap_or("");
        let start_pos = pos - current_word.len();
        let mut candidates = Vec::new();
        for cmd in &self.commands {
            if cmd.starts_with(current_word) {
                candidates.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone() + " ",
                })
            }
        }
        Ok((start_pos, candidates))
    }
}

const BUILTINS: &[&str] = &["exit", "echo", "type"];

enum Redirection {
    AppendStdout(String),
    AppendStderr(String),
    Stdout(String),
    Stderr(String),
}

struct CommandInfo {
    command: String,
    args: Vec<String>,
    redirection: Option<Redirection>,
}

impl CommandInfo {
    fn new(command: String, args: Vec<String>) -> Self {
        args.iter()
            .position(|s| s == "2>>")
            .map(|pos| {
                let redirect_file = args.get(pos + 1).cloned();
                if let Some(file) = redirect_file {
                    Self {
                        command: command.clone(),
                        args: args[..pos].to_vec(),
                        redirection: Some(Redirection::AppendStderr(file)),
                    }
                } else {
                    Self {
                        command: command.clone(),
                        args: args[..pos].to_vec(),
                        redirection: None,
                    }
                }
            })
            .or_else(|| {
                args.iter().position(|s| s == "2>").map(|pos| {
                    let redirect_file = args.get(pos + 1).cloned();
                    if let Some(file) = redirect_file {
                        Self {
                            command: command.clone(),
                            args: args[..pos].to_vec(),
                            redirection: Some(Redirection::Stderr(file)),
                        }
                    } else {
                        Self {
                            command: command.clone(),
                            args: args[..pos].to_vec(),
                            redirection: None,
                        }
                    }
                })
            })
            .or_else(|| {
                args.iter()
                    .position(|s| s == ">>" || s == "1>>")
                    .map(|pos| {
                        let redirect_file = args.get(pos + 1).cloned();
                        if let Some(file) = redirect_file {
                            Self {
                                command: command.clone(),
                                args: args[..pos].to_vec(),
                                redirection: Some(Redirection::AppendStdout(file)),
                            }
                        } else {
                            Self {
                                command: command.clone(),
                                args: args[..pos].to_vec(),
                                redirection: None,
                            }
                        }
                    })
            })
            .or_else(|| {
                args.iter().position(|s| s == ">" || s == "1>").map(|pos| {
                    let redirect_file = args.get(pos + 1).cloned();
                    if let Some(file) = redirect_file {
                        Self {
                            command: command.clone(),
                            args: args[..pos].to_vec(),
                            redirection: Some(Redirection::Stdout(file)),
                        }
                    } else {
                        Self {
                            command: command.clone(),
                            args: args[..pos].to_vec(),
                            redirection: None,
                        }
                    }
                })
            })
            .unwrap_or_else(|| Self {
                command,
                args,
                redirection: None,
            })
    }
}

fn main() {
    let path_env = std::env::var("PATH").unwrap_or(String::from(""));

    let mut rl = Editor::<MyHelper, DefaultHistory>::new().unwrap();
    let mut commands = Vec::from([String::from("echo"), String::from("exit")]);
    for dir in path_env.split(":"){
        let programs = search_path(Path::new(dir)).unwrap_or_default();
        for program in programs{
            if let Some(cmd) = program.file_name(){
                commands.push(cmd.to_string_lossy().to_string());
            }
        }
    }

    let helper = MyHelper {
        commands: commands,
    };
    rl.set_helper(Some(helper));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline("$ ");
        let mut inputs = String::new();

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                inputs = line.to_string();
                rl.save_history("history.txt");
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }

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
                    Some(Redirection::AppendStdout(file)) => {
                        let mut f = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(file)
                            .unwrap();
                        f.write_all(&content.into_bytes()).unwrap();
                    }
                    Some(Redirection::AppendStderr(file)) => {
                        let mut f = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(file)
                            .unwrap();
                        f.write_all(b"").unwrap();
                        print!("{}", content);
                    }
                    Some(Redirection::Stdout(file)) => write(file, content).unwrap(),
                    Some(Redirection::Stderr(file)) => {
                        write(file, "").unwrap();
                        print!("{}", content);
                    }
                    _ => print!("{}", content),
                }
            }
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
                    match command_info.redirection {
                        Some(Redirection::AppendStdout(file)) => {
                            let f = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(file)
                                .unwrap();
                            cmd.stdout(std::process::Stdio::from(f));
                        }
                        Some(Redirection::AppendStderr(file)) => {
                            let f = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(file)
                                .unwrap();
                            cmd.stderr(std::process::Stdio::from(f));
                        }
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
                    cmd.args(&command_info.args);
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
    let mut is_backslashed: bool = false;

    for ch in input.chars() {
        if !is_backslashed {
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
        let path = format!("{}/{}", dir, command);
        let meta = std::fs::metadata(&path).ok()?;
        if meta.permissions().mode() & 0o111 != 0 {
            Some(path)
        } else {
            None
        }
    })
}

fn read_input_line() -> Result<()> {
    // `()` can be used when no completer is required

    loop {}
    // #[cfg(feature = "with-file-history")]

    Ok(())
}

fn search_path(path: &Path) -> std::result::Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(path)? {
        let path = entry.unwrap().path();
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.permissions().mode() & 0o111 != 0 {
            files.push(path);
        }
    }
    Ok(files)
}
