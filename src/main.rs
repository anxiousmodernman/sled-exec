use sled;
use std::env;
use std::path;
use std::io;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::mem::size_of;

static USAGE: &str = r#"sled-exec - wrap a command and store the standard streams in a sled database

Usage:
    sled-exec [[OPTIONS] --] COMMAND [ARGS]

Options:
    --db PATH       Path to open or create sled database; Default: "sled-exec.db" in current directory
    --compress      Enable sled's log compression
"#;

fn main() -> Result<(), std::io::Error> {

    let mut args = env::args().skip(1); // skip "sled-exec", the first arg
    let mut config = sled::ConfigBuilder::new().path("sled-exec.db");
    let mut subcommand_args: Vec<String> = Vec::new();
    let mut more_conf_args = true;
    while let Some(arg) = args.next() {
        if more_conf_args {
            match arg.as_str() {
                "--" => {
                    more_conf_args = false;
                    continue;
                },
                "--db" => {
                    if let Some(path) = args.next() {
                        let path = path::Path::new(&path);
                        config = config.path(path);
                        continue;
                    } else {
                        exit_with_message(1, USAGE);
                    }
                },
                "--compress" => {
                    config = config.use_compression(true);
                    continue;
                }
                "-h" | "--help" => {
                    exit_with_message(1, USAGE);
                }
                _ => {
                    if arg.starts_with("-") {
                        exit_with_message(1, &format!("invalid argument: {}", arg));
                    }
                }
            }
            more_conf_args = false;
            continue;
        }
        subcommand_args.push(arg);
    }
    if subcommand_args.len() < 1 {
        exit_with_message(1, USAGE);
    }
    let mut iter = subcommand_args.iter();
    let mut cmd = std::process::Command::new(iter.next().unwrap());
    while let Some(arg) = iter.next() {
        cmd.arg(arg);
    }

    let mut tree = sled::Db::start(config.build()).expect("could not open database");

    let mut child = cmd.spawn()?;
    let mut child_stderr = BufReader::new(child.stderr.expect("child missing stderr"));
    let mut child_stdout = BufReader::new(child.stdout.expect("child missing stdout"));

    // collect our output into these buffers
    let mut line_stdout = Vec::new();
    let mut line_stderr = Vec::new();

    // Keep track of EOF on both streams
    let (mut eof_stdout, mut eof_stderr) = (false, false);

    // I guess this works for \r\n too.
    const NEWLINE: u8 = 0xA;

    loop {
        if !eof_stdout {
            match child_stdout.read_until(NEWLINE, &mut line_stdout) {
                Ok(n) if n == 0 => {
                    eof_stdout = true;
                }
                Ok(n) => {
                    let next_id = tree.generate_id().unwrap();
                    tree.set(format!("stdout:{:08}", next_id), line_stdout.clone()).unwrap().unwrap();
                }
                Err(e) => {
                    // There COULD be bytes in our buffer
                    eof_stdout = true;
                }
            };
            line_stdout.clear();
        }

        if !eof_stderr {
            match child_stderr.read_until(NEWLINE, &mut line_stderr) {
                Ok(n) if n == 0 => {
                    eof_stderr = true;
                }
                Ok(n) => {
                    let next_id = tree.generate_id().unwrap();
                    tree.set(format!("stderr:{:08}", next_id), line_stderr.clone()).unwrap().unwrap();
                }
                Err(e) => {
                    // There COULD be bytes in our buffer
                    eof_stderr = true;
                }
            };
            line_stderr.clear();
        }

        if eof_stderr && eof_stdout {
            break;
        }
    }

    Ok(())
}

fn exit_with_message(code: i32, msg: &str) {
    println!("{}", msg);
    std::process::exit(code);
}