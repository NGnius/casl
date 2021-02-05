use crate::speech::MetadataResult;
use crate::config::{Config, CommandConfig};
use std::process::{Command, Stdio};
use std::io::{BufWriter, BufReader, BufRead};
use crate::command_api::{Payload, Response, CommandAction};
use regex::{Regex, RegexBuilder};
use std::path::PathBuf;

const UDP_BUFFER_BYTES: usize = 8192;

pub trait ICommand {
    fn run(&self, input: &str);
}

pub fn process_commands(meta_result: &MetadataResult, casl_config: &Config) {
    if casl_config.debug {
        println!("Heard `{}` (processed into `{}`)", meta_result.phrase_raw, meta_result.phrase);
    }
    for cmd in &casl_config.commands {
        if cmd.use_raw() {
            if cmd.is_match(&meta_result.phrase_raw) {
                cmd.command().run(&meta_result.phrase_raw);
            }
        } else {
            if cmd.is_match(&meta_result.phrase) {
                cmd.command().run(&meta_result.phrase);
            }
        }
    }
}

// TODO
#[derive(Clone)]
pub struct SocketCommand {
    src_addr: String,
    dst_addr: String,
    src_port: usize,
    dst_port: usize,
}

impl SocketCommand {
    pub fn new(conf: &CommandConfig) -> SocketCommand {
        if let CommandConfig::Net {src_port, dst_port, src_addr, dst_addr, ..} = conf {
            SocketCommand {
                src_port: *src_port,
                dst_port: *dst_port,
                src_addr: src_addr.clone().unwrap_or("localhost".to_owned()),
                dst_addr: dst_addr.to_string(),
            }
        } else {panic!("Non-Net config given to SocketCommand");}
    }

    fn thread(input: String, src_addr: String, dst_addr: String, src_port: usize, dst_port: usize) {
        let socket = std::net::UdpSocket::bind(&format!("{}:{}", src_addr, src_port)).unwrap();
        let dst = format!("{}:{}", dst_addr, dst_port);
        let mut buf = [0; UDP_BUFFER_BYTES];
        // not TCP, but still set exclusive network address as target
        socket.connect(&dst)
            .expect(&format!("Failed to set destination address {}", &dst));
        // send payload
        let payload = Payload {
            text: input,
        };
        if socket.send(serde_json::to_string(&payload)
            .expect("Failed to serialize Payload").as_bytes())
            .is_err() {
            println!("Error sending UDP packet to {}, aborting SocketCommand", &dst);
            return;
        }
        // receive response
        let recv_result = socket.recv(&mut buf);
        if let Ok(length) = recv_result {
            let resp: Response = serde_json::from_slice(&buf[..length])
                .expect("Failed to deserialize Response");
            if let Some(err) = resp.error {
                println!("Command error received from {}: {}", &dst, &err);
                return;
            }
            // perform action
            resp.action.action().act();
        } else {
            println!("Error {} receiving UDP packet from {}, aborting SocketCommand", recv_result.err().unwrap(), &dst);
            return;
        }
    }
}

impl ICommand for SocketCommand {
    fn run(&self, input: &str) {
        let input_clone = input.clone().to_owned();
        let src_addr = self.src_addr.clone();
        let dst_addr = self.dst_addr.clone();
        let dst_port = self.dst_port;
        let src_port = self.src_port;
        std::thread::spawn(move || {
            Self::thread(input_clone, src_addr, dst_addr, src_port, dst_port);
        });
    }
}

// TODO
#[derive(Clone)]
pub struct StdIOCommand {
    command: String
}

impl StdIOCommand {
    pub fn new(conf: &CommandConfig) -> StdIOCommand {
        if let CommandConfig::StdIO { command, ..} = conf {
            StdIOCommand {
                command: command.clone(),
            }
        } else {panic!("Non-StdIO config given to StdIOCommand");}
    }

    fn thread(input: String, command: String) {
        let cmd = Command::new(&command)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn().expect("StdIOCommand thread failed to start");
        let stdin_writer = BufWriter::new(cmd.stdin.unwrap());
        let mut stdout_reader = BufReader::new(cmd.stdout.unwrap());
        // send payload
        serde_json::to_writer(stdin_writer, &Payload{
            text: input,
        }).expect("Failed to serialize Payload");
        // receive response
        let mut str_buf = String::new();
        stdout_reader.read_line(&mut str_buf).expect("Failed to read line of stdout");
        let resp: Response = serde_json::from_str(&str_buf)
            .expect("Failed to deserialize Response");
        // process response
        if let Some(err) = resp.error {
            println!("Command `{}` error: {}", &command, &err);
            return;
        }
        // perform action
        resp.action.action().act();
    }
}

impl ICommand for StdIOCommand {
    fn run(&self, input: &str) {
        let input_clone = input.clone().to_owned();
        let command = self.command.clone();
        std::thread::spawn(move || {
            Self::thread(input_clone, command);
        });
    }
}

// TODO
#[derive(Clone)]
pub struct ShellCommand {
    command: String,
    shell: String,
    precondition: Regex,
}

impl ShellCommand {
    pub fn new(conf: &CommandConfig) -> ShellCommand {
        if let CommandConfig::Shell { command, shell, precondition, ..} = conf {
            ShellCommand {
                command: command.clone(),
                shell: shell.clone(),
                precondition: RegexBuilder::new(precondition)
                    .case_insensitive(true)
                    .build().expect(&format!("Failed to compile the regex {} for shell command", precondition)),
            }
        } else {panic!("Non-Shell config given to ShellCommand");}
    }
}

impl ICommand for ShellCommand {
    fn run(&self, input: &str) {
        let mut str_buf = String::new();
        self.precondition.captures(input).unwrap().expand(&self.command, &mut str_buf);
        println!("Running {} command `{}`", &self.shell, &str_buf);
        std::process::Command::new(&self.shell)
            .arg("-c")
            .arg(&str_buf)
            .stdout(Stdio::null()) // TODO pipe stdio somewhere
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect(&format!("Failed to start {} command {}", &self.shell, &str_buf));
    }
}

pub struct RedirectCommand {
    command: Box<dyn ICommand>,
    path: PathBuf,
}

impl RedirectCommand {
    pub fn new(conf: &CommandConfig) -> RedirectCommand {
        if let CommandConfig::Redirect { path, ..} = conf {
            let json_file = std::fs::File::open(path).unwrap();
            let json_reader = std::io::BufReader::new(json_file);
            let conf: CommandConfig = serde_json::from_reader(json_reader)
                .expect(&("Unable to parse JSON file ".to_owned() + path));
            RedirectCommand {
                command: conf.command(),
                path: std::path::PathBuf::from(path),
            }
        } else {panic!("Non-Shell config given to ShellCommand");}
    }
}

impl ICommand for RedirectCommand {
    fn run(&self, input: &str) {
        self.command.run(input);
    }
}

impl Clone for RedirectCommand {
    fn clone(&self) -> Self {
        let json_file = std::fs::File::open(self.path.clone()).unwrap();
        let json_reader = std::io::BufReader::new(json_file);
        let conf: CommandConfig = serde_json::from_reader(json_reader).unwrap();
        RedirectCommand {
            command: conf.command(),
            path: std::path::PathBuf::from(self.path.clone()),
        }
    }
}

#[derive(Clone)]
pub struct AutoActionCommand {
    action: CommandAction,
}

impl AutoActionCommand {
    pub fn new(conf: &CommandConfig) -> AutoActionCommand {
        if let CommandConfig::Action { action, ..} = conf {
            AutoActionCommand {
                action: action.clone(),
            }
        } else {panic!("Non-Action config given to AutoActionCommand");}
    }
}

impl ICommand for AutoActionCommand {
    fn run(&self, _input: &str) {
        let action = self.action.clone();
        std::thread::spawn(move || {
            action.action().act();
        });
    }
}
