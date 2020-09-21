extern crate clap;
extern crate crypto;
extern crate rand;

use std::fs;
use std::io::{ErrorKind, Seek, SeekFrom};
use std::io::{Read, stdin, Write};
use std::process::exit;

use clap::{App, Arg, ArgMatches, SubCommand};
use crypto::{aes,blockmodes};
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use rand::RngCore;

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
        eprintln!("the -d/-e flag can't show up at same time");
        exit(-1);
    }
    let mut input = vec![0; 1024];
    let file_path = context.value_of("file").unwrap_or("");
    if file_path.is_empty() {
        eprintln!("need to specify the file path");
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
        handle_dec(&context, input);
    } else {
        handle_enc(&context, input)
    }
}

fn handle_enc(context: &ArgMatches, data: Vec<u8>) {
    if !context.is_present("o") {
        eprintln!("output file path hasn't been specified");
        exit(-1);
    }
    let file_path = context.value_of("o").unwrap_or("");
    if file_path.is_empty() {
        eprintln!("output file path hasn't been specified");
        exit(-1);
    }
    let mut file = fs::File::create(file_path).expect("create file failed");
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut iv);
    rng.fill_bytes(&mut key);
    file.write_all(&iv);
    file.write_all(&key);
    let mut buf = [0u8; 4096];
    let mut out = Vec::<u8>::new();
    let mut encryptor = aes::cbc_encryptor(aes::KeySize::KeySize256, &key, &iv, blockmodes::PkcsPadding);
    let mut read_buf = crypto::buffer::RefReadBuffer::new(&data);
    let mut write_buf = crypto::buffer::RefWriteBuffer::new(&mut buf);
    loop {
        let result = encryptor.encrypt(&mut read_buf, &mut write_buf, true).unwrap();
        out.extend(write_buf.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    file.write_all(&out);
}

fn handle_dec(context: &ArgMatches, data: Vec<u8>) {
    let iv = &data[0..16];
    let key = &data[16..48];
    let enc_data = &data[48..];
    let mut read_buf = crypto::buffer::RefReadBuffer::new(&enc_data);
    let mut buf = [0u8; 4096];
    let mut write_buf = crypto::buffer::RefWriteBuffer::new(&mut buf);
    let mut decryptor = aes::cbc_decryptor(aes::KeySize::KeySize256, &key, &iv, blockmodes::PkcsPadding);
    let mut out: Vec<u8> = Vec::new();
    loop {
        let result = decryptor.decrypt(&mut read_buf, &mut write_buf, true).unwrap();
        out.extend(write_buf.take_read_buffer().take_remaining().iter().map(|&i| { i }));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    if !context.is_present("o") {
        print!("{}", String::from_utf8(out).unwrap());
        return;
    }
    let file_path = context.value_of("o").unwrap_or("");
    let mut f = fs::File::open(file_path).unwrap_or_else(|e| {
        if e.kind() == ErrorKind::NotFound {
            match fs::File::create(file_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("{}", e);
                    exit(-1);
                }
            }
        } else {
            eprintln!("{}", e);
            exit(-1);
        }
    });
    f.write_all(&out);
}
