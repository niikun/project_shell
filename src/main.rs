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
        let mut parts = inputs.split_whitespace();
        let command = parts.next().unwrap_or("").to_string();
        let args:Vec<&str> = parts.collect();
        match command.trim() {
            "exit" => break,
            "echo" => println!("{}", args.join(" ")),
            "type" => {
                let arg_second = args.first().copied().unwrap_or("");
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
                                .args(&args)
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
