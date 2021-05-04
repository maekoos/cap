use minecraft_client_rs::Client;
use std::env;
use std::fs;
use std::io::{self, BufRead};

//TODO Get these from the arguments instead
static PASSWORD: &str = "password";
static ADDRESS: &str = "127.0.0.1:25575";

fn main() {
    println!("[info] Parsing commands");
    let cmds = parse_args();

    println!("[info] Connecting to server");
    let mut client = Client::new(ADDRESS.to_owned()).unwrap();
    match client.authenticate(PASSWORD.to_owned()) {
        Ok(_) => {
            println!("[info] Connected");
        }
        Err(_e) => {
            todo!("handle authentication error");
        }
    }

    println!("[info] Sending commands");
    for cmd in cmds {
        match client.send_command(cmd) {
            Ok(_resp) => {
                //TODO Log response?
                // println!("{}", resp.body);
            }
            Err(e) => {
                println!("Got error: {:?}", e);
            }
        }
    }

    println!("[info] Sent commands, disconnecting from server");
    client.close().unwrap();
}

fn parse_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    let input_file_path = args.get(1).expect("No input file.");

    let file = fs::File::open(input_file_path).unwrap();
    let reader = io::BufReader::new(file).lines();

    let mut out = Vec::new();
    for ln in reader {
        match ln {
            Ok(cmd) => out.push(cmd),
            Err(e) => panic!("Error reading lines: {}", e),
        }
    }

    out
}
