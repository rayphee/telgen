use std::fs::{canonicalize, File, OpenOptions, remove_file};
use std::io::{stdin, stdout, BufReader, Error, ErrorKind, Write};
use std::io::prelude::*;
use std::io;
use std::net::UdpSocket;
use std::path::Path;
use std::process::Command;
use std::process;

extern crate clap;
use clap::{Arg, App};
extern crate chrono;
use chrono::prelude::*;
extern crate whoami;
use whoami::username;

struct TelgenAgent {
    logfile: File,
    proc_id: u32,
    username: String,
}

impl TelgenAgent {
    fn parse_telgen_cmd(&mut self, cmd: &str) -> () {
        let mut parsed_cmd = cmd.split_whitespace();
        let telgen_cmd_0 = parsed_cmd.next();
        let telgen_kw = match telgen_cmd_0 {
            None => {return}, 
            _ => telgen_cmd_0.unwrap(), 
        };
        let command_line = cmd;
        match telgen_kw {
            "SPAWN" => {
                let telgen_cmd_1 = parsed_cmd.next();
                let spawn_cmd = match telgen_cmd_1 {
                    None => {
                        eprintln!("[WARNING]: No command specified");
                        return
                    }
                    _ =>telgen_cmd_1.unwrap(),
                };
                let cmd_args = parsed_cmd;

                let timestamp = Utc::now().to_rfc2822();
                let spawned_child = Command::new(spawn_cmd).args(cmd_args).spawn();
                let mut child = match spawned_child {
                    Err(s) => {
                        eprintln!("[WARNING]: {}", s);
                        return
                    },
                    Ok(_) => spawned_child.unwrap(),
                };
                let child_id = child.id();

                match self.log_common(timestamp, command_line, child_id, spawn_cmd, "SPAWN") {
                    Err(s) => eprintln!("[WARNING]: {}", s),
                    Ok(_) => (),
                }
                match child.wait() {
                    Err(s) => panic!("[ERROR]: Problem waiting on child: {}", s),
                    Ok(_) => (),
                }
            },
            "FILE" => {
                match self.file_op(command_line) {
                    Err(s) => {
                        eprintln!("[WARNING]: {}", s);
                    },
                    Ok(_) => (),
                }
            }
            "NET" => {
                match self.net_connect(command_line) {
                    Err(s) => {
                        eprintln!("[WARNING]: {}", s);
                    },
                    Ok(_) => (),
                }
            }
            &_ => {
                eprintln!("Not implemented");
            }
        }
    }

    fn file_op(&mut self, cmd: &str) -> io::Result<()> {
        let mut parsed_cmd = cmd.split_whitespace();
        let telgen_cmd_0 = parsed_cmd.next();
        match telgen_cmd_0 {
            None => {
                return Err(Error::new(ErrorKind::Other, "Unexpected operation"))
            }, 
            _ => telgen_cmd_0.unwrap(), 
        };
        let telgen_cmd_1 = parsed_cmd.next();
        let operation = match telgen_cmd_1 {
            None => {
                return Err(Error::new(ErrorKind::Other, "No FILE operation supplied"))
            },
            _ => telgen_cmd_1.unwrap(),
        };
        let telgen_cmd_2 = parsed_cmd.next();
        let filename = match telgen_cmd_2 {
            None => {
                return Err(Error::new(ErrorKind::Other, "No FILE filepath supplied"))
            },
            _ => telgen_cmd_2.unwrap(),
        };
        let telgen_cmd_3 = parsed_cmd.next();
        let data = match telgen_cmd_3 {
            None => "",
            _ => telgen_cmd_3.unwrap(),
        };
        let path = Path::new(&filename);
        match operation {
            "NEW" => {
                let timestamp = Utc::now().to_rfc2822();
                File::create(path)?;
                let path_buf = path.to_path_buf();
                let full_path = canonicalize(path_buf);
                self.log_common(timestamp, cmd, self.proc_id, "TELGEN", "FILE")?;
                writeln!(self.logfile, "    - file-operation:\"NEW\"")?;
                writeln!(self.logfile, "    - file-path:{:?}", full_path)?; 
            },
            "DEL" => {
                let path_buf = path.to_path_buf();
                let full_path = canonicalize(path_buf);
                let timestamp = Utc::now().to_rfc2822();
                remove_file(path)?;
                self.log_common(timestamp, cmd, self.proc_id, "TELGEN", "FILE")?;
                writeln!(self.logfile, "    - file-operation:\"DEL\"")?;
                writeln!(self.logfile, "    - file-path:{:?}", full_path)?;
            },
            "MOD" => {
                let timestamp = Utc::now().to_rfc2822();
                let mut file = OpenOptions::new().append(true).open(path)?; 
                write!(file, "{}", data)?;
                let path_buf = path.to_path_buf();
                let full_path = canonicalize(path_buf);
                self.log_common(timestamp, cmd, self.proc_id, "TELGEN", "FILE")?;
                writeln!(self.logfile, "    - file-operation:\"MOD\"")?;
                writeln!(self.logfile, "    - file-path:{:?}", full_path)?;
            },
            &_ => {
                return Err(Error::new(ErrorKind::Other, "FILE operation not implemented"))
            }
        }
        Ok(())
    }

    fn net_connect(&mut self, cmd: &str) -> io::Result<()> {
        let mut parsed_cmd = cmd.split_whitespace();
        let telgen_cmd_0 = parsed_cmd.next();
        match telgen_cmd_0 {
            None => {
                return Err(Error::new(ErrorKind::Other, "Unexpected operation"))
            }, 
            _ => telgen_cmd_0.unwrap(), 
        };
        let telgen_cmd_1 = parsed_cmd.next();
        let src = match telgen_cmd_1 {
            None => {
                return Err(Error::new(ErrorKind::Other, "No source IP address and port supplied"))
            },
            _ => telgen_cmd_1.unwrap(),
        };
        let telgen_cmd_2 = parsed_cmd.next();
        let dest = match telgen_cmd_2 {
            None => {
                return Err(Error::new(ErrorKind::Other, "No destination IP address and port supplied"))
            },
            _ => telgen_cmd_2.unwrap(),
        };
        let telgen_cmd_3 = parsed_cmd.next();
        let data = match telgen_cmd_3 {
            None => "",
            _ => telgen_cmd_3.unwrap(),
        };
        let timestamp = Utc::now().to_rfc2822();
        let socket = UdpSocket::bind(src)?;
        let bytes_sent = socket.send_to(data.as_bytes(), dest)?;
        self.log_common(timestamp, cmd, self.proc_id, "TELGEN", "NET")?;
        writeln!(self.logfile, "    - source:\"{}\"", src)?;
        writeln!(self.logfile, "    - destination:\"{}\"", dest)?;
        writeln!(self.logfile, "    - bytes-sent:\"{}\"", bytes_sent)?;
        writeln!(self.logfile, "    - protocol:\"UDP\"")
    }

    fn log_common(&mut self,
                  timestamp: String, 
                  command_line: &str, 
                  pid: u32, 
                  proc_name: &str, 
                  activity_type: &str) 
                  -> io::Result<()>{
        writeln!(self.logfile, "timestamp:{}", timestamp)?;
        writeln!(self.logfile, "  - command-line:\"{}\"", command_line)?;
        writeln!(self.logfile, "  - pid:{}", pid)?;
        writeln!(self.logfile, "  - process-name:\"{}\"", proc_name)?;
        writeln!(self.logfile, "  - username:\"{}\"", self.username)?;
        writeln!(self.logfile, "  - activity-type:\"{}\"", activity_type)
    }

}

fn console_prompt() -> () {
    print!("telgen> ");
    match stdout().flush() {
        Err(why) => panic!("Unable to flush stdout: {}", why),
        Ok(_) => (),
    }
}

fn main() {
    let matches = App::new("Telemetry Generator")
                       .version("0.1.0")
                       .author("Rafi Mueen")
                       .about("Generates and logs endpoint activity")
                       .arg(Arg::with_name("logfile")
                                .short("l")
                                .long("logfile")
                                .value_name("FILE")
                                .help("Sets file where endpoint activity is logged")
                                .takes_value(true))
                       .arg(Arg::with_name("SCRIPT")
                                .help("Script file to pull telemetry commands from")
                        )
                       .get_matches();

    let logfile_path = matches.value_of("logfile").unwrap_or("telemetry.log");
    eprintln!("Logging to {}", logfile_path);

    let stdin = stdin();
    let mut input = Box::new(stdin.lock()) as Box<dyn BufRead>;

    let script_file = matches.value_of("SCRIPT");
    let interactive_session: bool;
    match script_file {
        None => {
            interactive_session = true;
            console_prompt() 
        },
        _ => {
            interactive_session = false;
            let file = match File::open(script_file.unwrap()) {
                Err(why) => panic!("Unable to open scriptfile: {}", why),
                Ok(file) => file,
            };
            input = Box::new(BufReader::new(file));
        }
    }

    let mut telgen_agent = TelgenAgent {
        logfile: OpenOptions::new().create(true).append(true).open(logfile_path).unwrap(),
        proc_id: process::id(),
        username: username(),
    };

    // I'm not particularly happy about this; looking up constructors in Rust hasn't
    // shed light on the _proper_ way to initialize the logfile
    match writeln!(telgen_agent.logfile, "---") {
        Err(s) => panic!("Unable to write to logfile: {}", s),
        Ok(_) => (),
    }

    for line in input.lines(){

        let line = line.unwrap();
        let telgen_cmd = line.trim();
        telgen_agent.parse_telgen_cmd(&telgen_cmd);
        if interactive_session {console_prompt()};
    }
}
