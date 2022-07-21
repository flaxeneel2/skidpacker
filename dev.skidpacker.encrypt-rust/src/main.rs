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
    threads: usize,
    /// The key used to encrypt the classes
    #[clap(short, long, default_value="11111111111111111111111111111111")]
    key: String,
    /// The Nonce used to encrypt the classes
    #[clap(short, long, default_value="111111111111")]
    nonce: String
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

/// The main run function. It handles the args parsing as well as setting the number of threads that the program will be allowed to use.
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
    mass_encrypt_and_write_to_output_jar(classes, other_files);
}

/// Takes a vector of class names and a vector of resource names. Encrypts the classes and adds them to output jar, while only copying the resources in without encrypting them.
///
/// # Arguments
/// * `i_classes` - The initial classes. A vector of class names to be encrypted.
/// * `i_other` - The files that need to be left alone. Often just resources
fn mass_encrypt_and_write_to_output_jar(i_classes: Vec<String>, i_other: Vec<String>) {
    let start = SystemTime::now();
    let classes = i_classes.clone();
    let (tx, rx): (Sender<(String, Vec<u8>)>, _) = channel();
    let mut cs_hm: HashMap<String, Vec<u8>> = HashMap::new();
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let mut output_jar = ZipWriter::new(BufWriter::new(File::create(&args().output_jar).unwrap()));
    let mut key = args().key.clone();
    if key.len() != 32 {
        warn!("Key needs to be 32 characters long! Only the first 32 characters will be used and any missing characters will be filled with 1s");
        if key.len() > 32 {
            warn!("Removing excess characters from the key...");
            key = key[0..32].to_string();
        } else {
            warn!("Filling missing characters with 1s...");
            key.push_str("1".repeat(32-key.len()).as_str());
        }
    }
    log!("Key accepted!");
    let mut nonce = args().nonce.clone();
    if nonce.len() != 12 {
        warn!("Nonce needs to be 12 characters long! Only the first 12 characters will be used and any missing characters will be filled with 1s");
        if nonce.len() > 12 {
            warn!("Removing excess characters from nonce...");
            nonce = nonce[0..12].to_string();
        } else {
            warn!("Filling missing characters with 1s...");
            nonce.push_str("1".repeat(12-nonce.len()).as_str());
        }
    }
    log!("Nonce accepted!");
    let enc_data = (key.clone(), nonce.clone());
    for x in classes {
        let mut cb: Vec<u8> = Vec::new();
        z_jar.by_name(x.as_str()).unwrap().read_to_end(&mut cb).unwrap();
        cs_hm.insert(x, cb);
    }
    cs_hm.into_par_iter().for_each_with(tx, |tx, a| {
        let mut b = a.1;
        encrypt_class(&mut b, &a.0, enc_data.clone());
        tx.send((a.0, b)).expect("TODO: panic message");
    });
    for d in rx.iter() {
        output_jar.start_file(d.0, FileOptions::default()).expect("TODO: panic message");
        output_jar.write_all(&*d.1).expect("TODO: panic message");
    }
    i_other.iter().for_each(|a| {
        output_jar.raw_copy_file(z_jar.by_name(a.as_str()).unwrap()).unwrap();
    });
    output_jar.start_file("skidpackertest", FileOptions::default()).expect("Failed to create the test file");
    let mut test_data: Vec<u8> = b"Encryptionisprettygud".to_vec();
    raw_encrypt(&mut test_data, enc_data);
    output_jar.write_all(test_data.as_slice()).expect("failed to write test file data");
    if args().timings {
        log!(format!("Encryption done and encrypted jar generated! Time taken: {}ms", start.elapsed().unwrap().as_millis()))
    } else {
        log!("Encryption done and encrypted jar generated!")
    }
}

/// Just a raw encrypt function that adds no additional data and encrypts.
///
/// # Arguments
/// * `data` - The data to be encrypted
/// * `e` - The set of key and nonce to be used to encrypt.
fn raw_encrypt(data: &mut Vec<u8>, e: (String, String)) {
    let key = Key::from_slice(e.0.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(e.1.as_bytes());
    cipher.encrypt_in_place(nonce, b"", data).expect("Failed to encrypt");
}

/// Encrypt a class, adding the extra data such as name and length of the name to the final data
///
/// # Arguments
/// * `data` - The data to be encrypted
/// * `name` - The name of the class
/// * `e` - The set of key and nonce to be used to encrypt.
fn encrypt_class(data: &mut Vec<u8>, name: &String, e: (String, String)) {
    let key = Key::from_slice(e.0.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(e.1.as_bytes());
    let mut bytes = data.clone();
    cipher.encrypt_in_place(nonce, b"", &mut bytes).expect("Failed to encrypt");
    data.clear();
    data.push(name.clone().into_bytes().as_slice().len() as u8);
    data.extend_from_slice(name.clone().into_bytes().as_slice());
    data.extend_from_slice(bytes.as_slice());
}

/// This separates the contents of the jar file into classes and non-class files and places them into vectors that are passed by reference.
/// If it fails to read the zip archive, it will error and exit with code 1.
///
/// # Arguments
/// * `jar` - The jar whose contents need to be separated
/// * `class_vec` - The classes vector passed by reference that will be populated by the class names.
/// * `other_vec` - The non-class file vector passed by reference that will be populated with non-class files.
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