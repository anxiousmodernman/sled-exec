use sled;
use std::env;
use std::path;

static USAGE: &str = r#"sled-exec - wrap a command and store the standard streams in a sled database

Usage:
    sled-exec [[OPTIONS] --] COMMAND [ARGS]

Options:
    --db PATH       Path to open or create sled database; Default: "sled-exec.db" in current directory
    --compress      Enable sled's log compression
"#;

fn main() {

    let mut args = env::args().skip(1); // skip "sled-exec"
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

    let tree = sled::Db::start(config.build()).expect("could not open database");

}

fn exit_with_message(code: i32, msg: &str) {
    println!("{}", msg);
    std::process::exit(code);
}