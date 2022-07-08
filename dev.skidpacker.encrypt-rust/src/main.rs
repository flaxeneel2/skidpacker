use std::fs::File;
use std::path::Path;
use std::process::exit;
use colour::*;
use clap::Parser;
use once_cell::sync::OnceCell;
use zip::read::ZipFile;
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
    let mut classes: Vec<ZipFile> = Vec::new();
    let _output_jar = zip::write::ZipWriter::new(File::create(&args().output_jar).unwrap());
    get_classes(jar, &mut classes);

}

fn get_classes(jar: File, class_vec: &mut Vec<ZipFile>) {
    let data = ZipArchive::new(jar);
    let mut data = match data {
        Ok(f) => f,
        Err(err) => {
            error!(err);
            exit(1)
        }
    };
    for i in 0..data.len() {
        let file = match data.by_index(i) {
            Ok(f) => {f},
            Err(e) => {
                error!(e);
                exit(1)
            }
        };
        read_classes_recursively(file, class_vec)
    }
}

fn read_classes_recursively<'a>(z_file: ZipFile<'a>, class_vec: &mut Vec<ZipFile<'a>>) {
    
}

/*fn read_classes_recursively<'a>(z_file: ZipFile<'a>, class_vec: &mut Vec<ZipFile<'a>>) {
    if z_file.is_dir() {
        read_classes_recursively(z_file, class_vec)
    } else if z_file.name().ends_with(".class") {
        verbose!(format!("Queueing {}", z_file.name()));
        class_vec.push(z_file);
    } else {
        verbose!(format!("Skipping {}", z_file.name()));
    }
}*/

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