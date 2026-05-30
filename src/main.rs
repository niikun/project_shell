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
        let mut args: String = String::new();
        let mut command: String = String::new();
        io::stdin().read_line(&mut inputs).unwrap();
        for (i, input) in inputs.split_whitespace().enumerate() {
            if i == 0 {
                command = input.to_string();
            } else if i == 1 {
                args.push_str(input);
            } else {
                args.push_str(" ");
                args.push_str(input);
            }
        }
        match command.trim() {
            "exit" => break,
            "echo" => println!("{}", args),
            "type" => {
                let arg_second = args.split_whitespace().next().unwrap_or("");
                if BUILTINS.contains(&arg_second) {
                    println!("{} is a shell builtin", arg_second);
                } else {
                    let found =  path_env.split(":") .any(|path|{
                        let file_path = format!("{}/{}", path, arg_second);
                        if std::path::Path::new(&file_path).exists() {
                            let meta = std::fs::metadata(&file_path).unwrap();
                            if meta.permissions().mode() & 0o111 != 0 {
                                // println!("{:#o}",meta.permissions().mode());
                                println!("{} is {}", arg_second, &file_path);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    });
                    if !found{
                        println!("{}: not found", arg_second);
                    }
                }
            }
            _ => {
                let found:bool = path_env.split(":").any(|path|{
                    let file_path:String = format!("{}/{}",path,command);
                    if std::path::Path::new(&file_path).exists() {
                        let meta = std::fs::metadata(&file_path).unwrap();
                        if meta.permissions().mode() & 0o111 !=0 {
                            let mut child = Command::new(&command)
                                .args(args.split_whitespace())
                                .spawn()
                                .unwrap();
                            child.wait().unwrap();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });
                if !found{
                    println!("{}: not found", command);
                }
            },
        }
    }
}
