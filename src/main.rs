#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut msgs = String::new();
        let mut args: String = String::new();
        let mut command :String= String::new();
        io::stdin().read_line(&mut msgs).unwrap();
        for (i,msg) in msgs.split_whitespace().enumerate(){
            if i == 0{
                command = msg.to_string();
            } else if i==1 {
                args.push_str(msg);
            } else {
                args.push_str(" ");
                args.push_str(msg);
            }
        }
        match command.trim() {
            "exit" => break,
            "echo" =>println!("{}",args.as_str()),
            _ =>println!("{}: command not found",command.trim())
        }
    }
}
