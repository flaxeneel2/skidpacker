mod config;
mod macros;


use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc::{channel, Sender};
use aes_gcm::{AeadInPlace, Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{NewAead};
use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use crate::config::Config;
use once_cell::sync::OnceCell;
#[allow(unused)]
use colour::{blue_ln,white_ln,red_ln,yellow_ln};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rayon::ThreadPoolBuilder;

use zip::ZipArchive;

/*lazy_static! {
    pub static ref JNI_PTR: Mutex<usize> = Mutex::new(0);
    pub static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}*/

static JNI_PTR: OnceCell<usize> = OnceCell::new();
static CONFIG: OnceCell<Config> = OnceCell::new();

/// The init function that has to be run to get the JNI pointer
/// # Arguments
/// * `env` - The JNI env pointer
/// * `_class` - Unused
/// * `configPath` - Path to the config file to load
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_dev_skidpacker_loader_jni_init(env: *mut u8, _class: JClass, configPath: JString) {
    JNI_PTR.set(env as usize).unwrap();
    let cfg_path: String = get_jni_env().get_string(configPath).unwrap().into();
    let cfg: Config = Config::load(cfg_path.as_str());
    ThreadPoolBuilder::new().num_threads(cfg.threads).build_global().unwrap();
    CONFIG.set(cfg).unwrap();
    let jar = get_jar();
    test_jar(&jar);
    load_jar(jar);
}

fn load_jar(jar: File) {
    let mut classes: Vec<String> = Vec::new();
    let mut resources: Vec<String> = Vec::new();
    separate_classes(&mut classes, &mut resources, &jar);
    decrypt_and_load(&mut classes);
    load_resources(&mut resources);
}

fn separate_classes(classes: &mut Vec<String>, resources: &mut Vec<String>, jarfile: &File) {
    let data = match ZipArchive::new(jarfile) {
        Ok(f) => f,
        Err(err) => {
            error!(err);
            exit(1)
        }
    };
    for f_name in data.file_names() {
        if f_name.ends_with(".class") {
            classes.push(f_name.to_string())
        } else { resources.push(f_name.to_string()) }
    }
}

fn get_loader() -> JObject<'static> {
    let class_loader_class = get_jni_env().find_class("java/lang/ClassLoader").unwrap();
    get_jni_env().call_static_method(class_loader_class, "getSystemClassLoader", "()Ljava/lang/ClassLoader;", &[]).unwrap().l().unwrap()
}

fn decrypt_and_load(class_names: &mut Vec<String>) {
    let loader = get_loader();

    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let cs_hm: HashMap<String, Vec<u8>> = HashMap::new();
    for cn in class_names {
        let mut cb: Vec<u8> = Vec::new();
        z_jar.by_name(cn).unwrap().read_to_end(&mut cb).unwrap();
    }
    let (tx, rx): (Sender<(String, Vec<u8>)>, _) = channel();
    cs_hm.par_iter().for_each_with(tx, |tx, a| {
        let mut d = a.1.clone();
        strip_name_data_from_class_bytes(&mut d);
        decrypt_class_bytes(&mut d);
        tx.send((a.0.clone(), d)).unwrap();
    });
    for d in rx.iter() {
        get_jni_env().define_class(d.0, loader, d.1.as_slice()).unwrap();
    }
}


fn load_resources(resources: &mut Vec<String>) {
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let mut r_hm: HashMap<String, Vec<u8>> = HashMap::new();
    for resource in resources {
        let mut res_bytes: Vec<u8> = Vec::new();
        z_jar.by_name(resource).unwrap().read_to_end(&mut res_bytes).unwrap();
        r_hm.insert(resource.clone(), res_bytes);
    }
    r_hm.par_iter().for_each(|a| {
        let loader = get_loader();
        verbose!(format!("would have loaded class: {} with {} bytes", a.0, a.1.len()));
        get_jni_env().call_method(loader, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[get_jni_env().new_string(a.0).unwrap().into(), get_jni_env().byte_array_from_slice(a.1).unwrap().into()]).unwrap();
    })
}

/// Decrypts an encrypted class and returns byte vector
/// # Arguments
/// * `class_name` - Name of the class to be decrypted.
///
fn decrypt_class_bytes(class_data: &mut Vec<u8>) {
    let key = Key::from_slice(config().license.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"THISISANONCE");
    cipher.decrypt_in_place(nonce, b"",class_data).unwrap();
}

/// As the very long but descriptive name suggests, this function strips the name data from the stored class bytes.
/// NOTE: THIS DOES NOT CHECK IF THE CLASS IS A PRE-STRIPPED CLASS OR NOT. PLEASE USE CAREFULLY
///
/// # Arguments
/// * `class_bytes` - The class byte vector to strip the name data from
fn strip_name_data_from_class_bytes(class_bytes: &mut Vec<u8>) {
    let cuts = class_bytes.get(0).unwrap().clone() as usize;
    class_bytes.drain(0..cuts);
}

/// Get the class name from the class bytes
/// Currently unused but may be used in the future iterations of this loader
///
/// # Arguments
/// * `class_bytes` - The bytes of the class file
#[allow(unused)]
fn get_class_name(class_bytes: Vec<u8>) -> String {
    let length = *class_bytes.get(0).unwrap() as usize;
    let name_slice = String::from_utf8(class_bytes[1..length].to_vec()).unwrap();
    name_slice
}

/// Get the JNI env
/// The value from the JNI PTR is used as a raw pointer
fn get_jni_env() -> JNIEnv<'static> {
    let ptr = JNI_PTR.get().unwrap().clone();
    unsafe { JNIEnv::from_raw(ptr as *mut _).unwrap() }
}

fn get_jar() -> File {
    let name = config().input_jar.clone();
    if !Path::exists(Path::new(&name)) {
        error!("Input jar not found!");
        exit(0);
    }
    let jar = File::open(name);
    if jar.is_err() {
        error!(format!("{}", jar.unwrap_err()));
        exit(0);
    }
    jar.unwrap()
}

fn test_jar(jar: &File) {
    let mut z_jar = ZipArchive::new(jar).unwrap();
    let mut d = Vec::new();
    let a = z_jar.by_name("skidpackertest");
    if a.is_err() {
        error!("The jar that you wanted to load doesn't seem to be a skidpacked jar!");
        exit(1)
    }
    let mut a = a.unwrap();
    let e = a.read_to_end(&mut d);
    if e.is_err() {
        error!("Failed to read test file! exiting...");
        exit(1)
    }
    decrypt_class_bytes(&mut d);
    let m = String::from_utf8(d);
    if m.is_err() {
        error!("Invalid key! Exiting...");
        exit(1);
    }
    let ans = m.unwrap();
    if ans != "Encryptionisprettygud" {
        error!("Invalid key! Exiting...");
        exit(1);
    }
}

fn config() -> &'static Config {
    CONFIG.get().unwrap()
}


