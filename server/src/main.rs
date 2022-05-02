#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unused_assignments)]

// Configuration processing
use configparser::ini::Ini;
// Networking
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

// Command Handling
use std::process::Command;
// URL Processing
use std::collections::HashMap;
use url::Url;

mod parser {
    use std::collections::HashMap;
    use std::env;

    fn parse_equal(cmd: &str) -> (String, String) {
        let s = cmd.split("=").collect::<Vec<&str>>();
        return (
            String::from(s[0].clone()).replace("-", ""),
            String::from(s[1].clone()),
        );
    }
    fn parse_normal(cmd: (String, String)) -> (String, String) {
        return (
            String::from(cmd.0.clone()).replace("-", ""),
            String::from(cmd.1.clone()),
        );
    }

    pub fn parse() -> HashMap<String, String> {
        let mut args: Vec<String> = env::args().collect();
        args.remove(0);

        let mut args_hash: HashMap<String, String> = HashMap::new();

        let mut i: usize = 0;
        while i < args.len() {
            let cmd = &args.clone()[i];
            if cmd.find("=").is_some() {
                let res = parse_equal(&cmd);
                args_hash.insert(res.0, res.1);
            } else {
                if i + 1 >= args.len() {
                    let res = parse_normal((cmd.clone(), String::from("true")));
                    args_hash.insert(res.0, res.1);
                    break;
                } else {
                    if args[i + 1].starts_with("-") {
                        let res = parse_normal((cmd.to_string(), String::from("true")));
                        args_hash.insert(res.0, res.1);
                    } else {
                        let res = parse_normal((cmd.to_string(), args[i + 1].to_string()));
                        args_hash.insert(res.0, res.1);
                        i += 1;
                    }
                }
            }
            i += 1;
        }
        return args_hash;
    }
}

struct OsteConfig {
    ip: String,
    port: String,
    password: String,
}
impl OsteConfig {
    fn new() -> Self {
        Self {
            ip: String::new(),
            password: String::new(),
            port: String::new(),
        }
    }
}
fn get_config(config_path: &str) -> Result<OsteConfig, Box<dyn std::error::Error>> {
    let mut config = Ini::new();
    let mut _map = config.load(config_path)?;
    let map = _map["default"].clone();
    assert_ne!(map.get("ip"), None, "IP Field doesn't exist");
    let ip = map.get("ip").clone().unwrap().clone().unwrap();
    assert_ne!(map.get("port"), None, "PORT Field doesn't exist");
    let port = map.get("port").clone().unwrap().clone().unwrap();
    assert_ne!(map.get("password"), None, "PASSWORD Field doesn't exist");
    let password = map.get("password").clone().unwrap().clone().unwrap();

    return Ok(OsteConfig { ip, port, password });
}

fn help() {
    println!(
        "usage: osteserver.exe [-c,--config] [-h,--help]
    -c,--config : Custom configuration file (default : config.ini)
    -h,--help : shows this menu
    "
    );
}

fn main() {
    let mut config_path: &str = "config.ini";
    // Args parsing
    let args: HashMap<String, String> = parser::parse();
    if (args.contains_key("config")) {
        config_path = args.get("config").unwrap();
        println!("Loading {} as a config file", config_path);
    }
    if (args.contains_key("help")) {
        println!(
            "osteserver.exe [OPTIONS]
        --config : a custom configuration file (default: config.ini)
        --help : show this menu
        "
        );
        return;
    }
    let osteconfig: Result<OsteConfig, Box<dyn std::error::Error>> = get_config(&config_path);
    let OsteConfig { ip, port, password } = osteconfig.unwrap();
    let server = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(server).unwrap();
    {
        println!("Web Client Settings\n");
        println!("==================================");
        println!("Server's IP : {}", ip);
        println!("Server's Port : {}", port);
        println!("==================================");
    }
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, &password);
    }
}

fn handle_connection(mut stream: TcpStream, password: &String) {
    // Reading stream
    let mut buffer = [0; 2048];
    let mut response: String = String::from("");
    stream.read(&mut buffer).unwrap();
    // Parsing Headers
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    let parsed_buffer = req.parse(&buffer[..]);
    if (parsed_buffer.is_err()) {
    } else {
        let res = parsed_buffer.unwrap();
        assert_eq!(res.is_partial(), false, "Is partial indeed");
        // Parsing URL Queries
        let url_data = Url::parse(format!("http://localhost{}", req.path.unwrap()).as_str())
            .expect("Error while parsing URL");
        let mut queries: HashMap<String, String> = HashMap::new();
        // Grabbing query pairs
        let mut query = url_data.query_pairs();
        // Looping through them
        for i in 0..query.count() {
            let (k, v): (std::borrow::Cow<str>, std::borrow::Cow<str>) = query.next().unwrap();
            queries.insert(String::from(k), String::from(v));
        }
        if url_data.path().starts_with("/command") && req.method.unwrap() == "GET" {
            let pwd = queries.get("password");
            let cmd = queries.get("cmd");
            if (pwd.is_none() || cmd.is_none()) {
                let msg = "One of the arguments is missing";
                response = String::from(format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{}",
                    msg.len(),
                    msg
                ));
            } else {
                if (*password != *pwd.unwrap()) {
                    response = String::from(
                        "HTTP/1.1 401 Unauthorized\r\nContent-Length: 12\r\n\r\nUnauthorized",
                    )
                } else {
                    let output = if cfg!(target_os = "windows") {
                        Command::new("powershell")
                            .args(["-Command", cmd.unwrap()])
                            .output()
                            .expect("failed to execute command")
                    } else {
                        Command::new("sh")
                            .args(["-c", cmd.unwrap()])
                            .output()
                            .expect("failed to execute command")
                    };
                    let out = String::from_utf8(output.stdout);
                    let err = String::from_utf8(output.stderr);
                    if (out.clone().unwrap().len() > 0) {
                        response = String::from(format!(
                            "HTTP/1.1 200 OK \r\nContent-Length: {}\r\n\r\n{}",
                            out.clone().unwrap().len(),
                            out.clone().unwrap()
                        ));
                    } else if (err.clone().unwrap().len() > 0) {
                        let error = err.unwrap();
                        response = String::from(format!(
                            "HTTP/1.1 500 Internal Server Error \r\nContent-Length : {}\r\n\r\n{}",
                            error.len(),
                            error
                        ))
                    } else {
                        response = String::from("HTTP/1.1 200 OK \r\n");
                    }
                }
            }
        } else if (url_data.path().starts_with("/echo") && req.method.unwrap() == "GET") {
            response = String::from("HTTP/1.1 200 OK\r\n\r\n");
        } else {
            response = String::from("HTTP/1.1 404 NOT FOUND\r\n\r\n");
        }
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
