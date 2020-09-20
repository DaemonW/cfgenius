extern crate clap;

use std::fs;

use clap::{App, Arg, SubCommand};
use std::process::exit;
use std::io::{stdin, Read};

fn main() {
    let context = App::new("cfgenius")
        .version("0.5.0")
        .author("develop by daemonw")
        .about("a config encode/decode tool")
        .arg(Arg::with_name("d")
            .short("d")
            .help("decode a config file"))
        .arg(Arg::with_name("e")
            .short("e")
            .help("encode a config file"))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("specify the file path to be handled")
            .takes_value(true))
        .arg(Arg::with_name("o")
            .short("o")
            .takes_value(true)
            .value_name("FILE")
            .help("specify the output file, defaut is stdout"))
        .get_matches();
    let cmd_enc = context.is_present("e");
    let cmd_dec = context.is_present("d");
    if cmd_enc && cmd_dec {
        eprintln!("the decode and encode flag can't show up at same time");
        exit(-1);
    }
    let mut input = vec![0; 1024];
    let file_path = context.value_of("file").unwrap_or("");
    if file_path.is_empty() {
        eprintln!("you need to specify the file path");
        exit(-1);
        return;
    }
    if !file_path.is_empty() {
        match fs::read(file_path) {
            Ok(data) => {
                input = data;
            }
            Err(e) => {
                println!("{}", e);
                exit(-1);
            }
        }
    }
    if cmd_dec {
        handle_dec(input);
    } else {
        handle_enc(input)
    }
}

fn handle_enc(data: Vec<u8>) {
    let str = String::from_utf8(data).unwrap();
    println!(str);
}

fn handle_dec(data: Vec<u8>) {}
