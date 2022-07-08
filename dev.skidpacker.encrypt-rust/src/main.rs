
use std::fs::File;
use std::io::{Write};

use std::path::Path;
use std::process::exit;
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
    output_jar: String
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
    verbose!("Arguments accepted!");
    log!(format!("loading {}", &args().input_jar));
    let jar = get_jar();
    encrypt_jar(jar);
}

fn encrypt_jar(jar: File) {
    let mut classes: Vec<String> = Vec::new();
    let mut output_jar = zip::write::ZipWriter::new(File::create(&args().output_jar).unwrap()); //not in use for now
    get_classes(jar.try_clone().unwrap(), &mut classes);
    let mut z_jar = ZipArchive::new(jar).unwrap();
    for class in classes {
        let clazz = z_jar.by_name(class.as_str()).unwrap();
        output_jar.start_file(clazz.name(), FileOptions::default()).unwrap();
        output_jar.write_all(format!("File name: {}, Size: {}, Compressed Size: {}", clazz.name(), clazz.size(), clazz.compressed_size()).as_bytes()).expect("TODO: panic message");
        verbose!(format!("File name: {}, Size: {}, Compressed Size: {}", clazz.name(), clazz.size(), clazz.compressed_size()))

    }
}

fn encrypt_class(data: &[u8]) {

}

fn get_classes(jar: File, class_vec: &mut Vec<String>) {
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
            verbose!(format!("Rejected: {}", i));
            num_rejected+=1;
        }
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