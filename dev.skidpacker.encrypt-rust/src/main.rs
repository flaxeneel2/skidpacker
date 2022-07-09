extern crate core;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use aes_gcm::{AeadInPlace, Aes256Gcm, Key, Nonce}; // Or `Aes128Gcm`
use aes_gcm::aead::{NewAead};
use colour::*;
use clap::Parser;
use once_cell::sync::OnceCell;

use zip::write::FileOptions;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The jar to encrypt.
    #[clap(short, long, default_value="input.jar")]
    input_jar: String,
    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
    /// The name of the output jar
    #[clap(short, long, default_value="output.jar")]
    output_jar: String,
    /// Report the time taken to complete the task
    #[clap(short, long)]
    timings: bool
}

static ARGS: OnceCell<Args> = OnceCell::new();

/// Print function for verbose output
/// # Arguments
/// * `msg` - The message to output
macro_rules! verbose {
    ($msg: expr) => {
        if args().verbose { white_ln!("DEBUG: {}", $msg) }
    };
}

/// Print function for warning output
/// # Arguments
/// * `msg` - The message to output
macro_rules! warn {
    ($msg: expr) => {
        yellow_ln!("WARN:  {}", $msg)
    };
}

/// Print function for error output
/// # Arguments
/// * `msg` - The message to output
macro_rules! error {
    ($msg: expr) => {
        red_ln!("ERROR: {}", $msg)
    };
}

/// Print function for just basic log output
/// # Arguments
/// * `msg` - The message to output
macro_rules! log {
    ($msg: expr) => {
        blue_ln!("LOG:   {}", $msg)
    };
}


fn main() {
    let loaded_args = Args::parse(); //parse the args
    ARGS.set(loaded_args).unwrap();
    let start = std::time::SystemTime::now();
    if args().timings {
        log!("Timer started");
    }
    verbose!("Arguments accepted!");
    log!(format!("loading {}", &args().input_jar));
    let jar = get_jar();
    encrypt_jar(jar);
    log!(format!("{} has been encrypted! Encrypted jar saved as {}", args().input_jar, args().output_jar));
    if args().timings {
        let end = start.elapsed().unwrap();
        log!(format!("Entire operation finished! Time taken: {}ms", end.as_millis()));
    }
}

/// The main entrypoint for the encryption process
///
/// # Arguments
/// * `jar` - The jar to encrypt
fn encrypt_jar(jar: File) {
    let mut classes: Vec<String> = Vec::new();
    let mut other_files: Vec<String> = Vec::new();
    let mut output_jar = zip::write::ZipWriter::new(File::create(&args().output_jar).unwrap()); //not in use for now

    separate_classes(jar.try_clone().unwrap(), &mut classes, &mut other_files);

    let mut z_jar = ZipArchive::new(jar).unwrap();

    log!("Classes and files loaded! Encrypting classes...");

    for class in classes {
        let mut clazz = z_jar.by_name(class.as_str()).unwrap();
        let mut clazz_bytes: Vec<u8> = Vec::new();

        clazz.read_to_end(&mut clazz_bytes).expect("Unable to read class bytes!");
        output_jar.start_file(clazz.name(), FileOptions::default()).unwrap();

        encrypt_class(&mut clazz_bytes);

        output_jar.write_all(clazz_bytes.as_slice()).expect("TODO: panic message");
        verbose!(format!("Encrypted and added class {}", clazz.name()))
    }

    log!("Classes encrypted!");

    for other in other_files {
        let mut file = z_jar.by_name(other.as_str()).unwrap();
        let mut file_bytes: Vec<u8> = Vec::new();

        file.read_to_end(&mut file_bytes).expect("Unable to read file bytes!");
        output_jar.start_file(file.name(), FileOptions::default()).unwrap();
        output_jar.write_all(file_bytes.as_slice()).expect("Unable to write file to output jar!");

        verbose!(format!("Added file {}", file.name()));
    }
    log!("Files added!")
}

fn encrypt_class(data: &mut Vec<u8>) {
    let key = Key::from_slice(b"11111111111111111111111111111111");
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice("111111111111".as_bytes());
    cipher.encrypt_in_place(nonce, b"", data).expect("Failed to encrypt");
}

fn separate_classes(jar: File, class_vec: &mut Vec<String>, other_vec: &mut Vec<String>) {
    let data = match ZipArchive::new(jar) {
        Ok(f) => f,
        Err(err) => {
            error!(err);
            exit(1)
        }
    };
    let mut num_accepted = 0;
    let mut num_rejected = 0;
    let f_names = data.file_names();
    for i in f_names {
        verbose!(format!("Found file: {}", i));
        if i.ends_with(".class") {
            class_vec.push(i.to_string());
            verbose!(format!("Accepted class: {}", i));
            num_accepted+=1;
        } else {
            other_vec.push(i.to_string());
            verbose!(format!("Rejected: {}", i));
            num_rejected+=1;
        }
    }
    if num_accepted==0 {
        warn!("No classes detected! Please run with `-v` to see the list of accepted and rejected files")
    }
    log!(format!("Jar read finished! {} Accepted and {} Rejected", num_accepted, num_rejected))
}

/// Get the jarfile to be encrypted
/// This will error and exit the program if the file does not exist or there was an error reading the file.
fn get_jar() -> File {
    if !Path::new(args().input_jar.as_str()).exists() {
        error!("Jar does not exist!");
        exit(1)
    }
    let f = File::open(&args().input_jar);
    if f.is_err() {
        error!(format!("{}", f.unwrap_err()));
        exit(1)
    }
    f.unwrap()
}

/// Get the args
fn args() -> &'static Args {
    ARGS.get().unwrap()
}