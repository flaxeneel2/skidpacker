extern crate core;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::process::exit;

use std::sync::mpsc::{channel, Sender};
use std::time::{SystemTime};
use aes_gcm::{AeadInPlace, Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{NewAead};
use colour::*;
use clap::Parser;
use once_cell::sync::OnceCell;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use rayon::ThreadPoolBuilder;


use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

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
    timings: bool,
    /// Number of threads to run the encryption on
    #[clap(short='T', long, default_value_t=4)]
    threads: usize
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
    let start = SystemTime::now();
    if args().timings {
        log!("Timer started");
    }
    ThreadPoolBuilder::new().num_threads(args().threads).build_global().unwrap();
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

    separate_classes(jar.try_clone().unwrap(), &mut classes, &mut other_files);
    let mut encrypted_classes: HashMap<String, Vec<u8>> = HashMap::new();
    fire_threads(classes, &mut encrypted_classes);
    let mut output_jar = ZipWriter::new(BufWriter::new(File::create(&args().output_jar).unwrap()));
    for m in encrypted_classes.iter() {
        let class_name = m.0.to_owned();
        let class_data = m.1.to_owned();
        output_jar.start_file(class_name, FileOptions::default()).unwrap();
        output_jar.write_all(&*class_data).unwrap();
    }
}

fn fire_threads(i_classes: Vec<String> ,f_hashmap: &mut HashMap<String, Vec<u8>>) {
    let start = SystemTime::now();
    let classes = i_classes.clone();
    let (tx, rx): (Sender<(String, Vec<u8>)>, _) = channel();
    let mut cs_hm: HashMap<String, Vec<u8>> = HashMap::new();
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    for x in classes {
        let mut cb: Vec<u8> = Vec::new();
        z_jar.by_name(x.as_str()).unwrap().read_to_end(&mut cb).unwrap();
        cs_hm.insert(x, cb);
    }
    cs_hm.into_par_iter().for_each_with(tx, |tx, a| {
        let mut b = a.1;
        encrypt_class(&mut b);
        tx.send((a.0, b)).unwrap();
    });
    for d in rx.iter() {
        f_hashmap.insert(d.0, d.1);
    }
    log!(format!("Encryption Done! Time taken: {}ms", start.elapsed().unwrap().as_millis()))
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