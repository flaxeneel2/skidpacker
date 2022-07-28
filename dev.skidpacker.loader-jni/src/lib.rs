mod config;
mod macros;


use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use std::path::Path;
use std::process::exit;
use std::sync::{RwLock};
use std::sync::mpsc::{channel, Sender};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get};

use actix_web::http::StatusCode;
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
use serde_json::json;

use zip::ZipArchive;


/// The once cell for the JNI pointer
static JNI_PTR: OnceCell<usize> = OnceCell::new();
/// The once cell for the config.
static CONFIG: OnceCell<Config> = OnceCell::new();

static NONCE: &str = "THISISANONCE";

static CLASS_COUNT: OnceCell<RwLock<i32>> = OnceCell::new();

/// The init function that has to be run to get the JNI pointer
/// # Arguments
/// * `env` - The JNI env pointer
/// * `_class` - Unused
/// * `configPath` - Path to the config file to load
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_dev_skidpacker_loader_Jni_init(env: *mut u8, _class: JClass, configPath: JString, j_args: JObject) {
    JNI_PTR.set(env as usize).unwrap();
    log!("Loader dll loaded!");
    let cfg_path: String = get_jni_env().get_string(configPath).unwrap().into();
    let cfg: Config = Config::load(cfg_path.as_str());
    ThreadPoolBuilder::new().num_threads(cfg.threads).build_global().unwrap();
    CONFIG.set(cfg).unwrap();
    CLASS_COUNT.set(RwLock::new(0)).unwrap();
    let jar = get_jar();
    test_jar(&jar);
    rayon::spawn(|| {
        log!("Starting webserver...");
        webserver();
        log!("Webserver started!");
    });
    load_jar(jar, j_args);
}

/// The main load function. This creates the classes and resources vectors to be passed by reference
/// and splits into sub-functions that are then run.
///
/// # Arguments
/// * `jar` - The jar to load
fn load_jar(jar: File, args: JObject) {
    let mut classes: Vec<String> = Vec::new();
    let mut resources: Vec<String> = Vec::new();
    separate_classes(&mut classes, &mut resources, &jar);
    decrypt_and_load(&mut classes, args);
    //load_resources(&mut resources);
}

/// This function reads the jar to be loaded and separates its classes and resources into two separate vectors.
///
/// # Arguments
/// * `classes` - The classes vector passed by reference. This is populated with the classnames.
/// * `resources` - The resources vector passed by reference. This is populated with the names of the resources.
/// * `jarfile` - The jar whose classes and resources need to be separated.
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

/// Get the loader object to load classes and resources.
fn get_loader() -> JObject<'static> {
    let class_loader_class = get_jni_env().find_class("java/lang/ClassLoader").unwrap();
    get_jni_env().call_static_method(class_loader_class, "getSystemClassLoader", "()Ljava/lang/ClassLoader;", &[]).unwrap().l().unwrap()
}

/// The decrypt and load function. This function decrypts the classes using the number of threads specified in the config and loads them
/// into the JVM.
///
/// # Arguments
/// * `class_names` - Names of the classes to be loaded.
fn decrypt_and_load(class_names: &mut Vec<String>, args: JObject) {
    let loader = get_loader();
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let mut cs_hm: HashMap<String, Vec<u8>> = HashMap::new();
    for cn in class_names {
        let mut cb: Vec<u8> = Vec::new();
        z_jar.by_name(cn).unwrap().read_to_end(&mut cb).unwrap();
        cs_hm.insert(cn.to_owned(), cb);
    }
    let (tx, rx): (Sender<(String, Vec<u8>)>, _) = channel();
    cs_hm.par_iter().for_each_with(tx, |tx, a| {
        let mut d = a.1.clone();
        let mut n = a.0.clone();
        n = n.replace(".class", "");
        n = n.replace(".", "/");
        strip_name_data_from_class_bytes(&mut d);
        decrypt_class_bytes(&mut d);
        tx.send((n, d)).unwrap();
    });
    for a in rx.iter() {
        verbose!(format!("Loading {}!", a.0));
        let ae = get_jni_env().define_class(a.0, loader, a.1.as_slice());
        if ae.is_err() {
            error!("Error defining class!");
            error!(format!("{:?}", ae.unwrap_err()));
            get_jni_env().exception_describe().unwrap();
            exit(1);
        }
        increment_web_class_count();
    }
    run_entrypoint_func(args);
}

/// This function loads the non-class files (resources) into the resource cache.
///
/// # Arguments
/// * `resources` - The vector of non-class file names to be loaded.
fn _load_resources(resources: &mut Vec<String>) {
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let mut r_hm: HashMap<String, Vec<u8>> = HashMap::new();
    for resource in resources {
        let mut res_bytes: Vec<u8> = Vec::new();
        z_jar.by_name(resource).unwrap().read_to_end(&mut res_bytes).unwrap();
        r_hm.insert(resource.clone(), res_bytes);
    }
    r_hm.par_iter().for_each(|a| {
        let loader = get_loader();
        get_jni_env().call_method(loader, "put", "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;", &[get_jni_env().new_string(a.0).unwrap().into(), get_jni_env().byte_array_from_slice(a.1).unwrap().into()]).unwrap();
    })
}

fn run_entrypoint_func(args: JObject/*, class: JClass*/) {
    let class_name = get_entrypoint_class().trim().to_string();
    let class_name = class_name.replace(".class", "");
    let class_name = class_name.replace(".", "/");
    let class = match get_jni_env().find_class(class_name) {
        Ok(v) => v,
        Err(e) => {
            error!("Unable to get main class via JNI!");
            error!(format!("{:?}", e));
            get_jni_env().exception_describe().unwrap();
            exit(1);
        }
    };
    let n = get_jni_env().call_static_method(class, "main", "([Ljava/lang/String;)V", &[args.into()]);
    match n {
        Ok(_) => { log!("Main method called!") },
        Err(e) => {
            error!(format!("{:?}", e));
            error!(format!("{:?}", get_jni_env().exception_occurred().unwrap().into_inner()));
        }
    }
}

fn get_entrypoint_class() -> String {
    let mut z_jar = ZipArchive::new(get_jar()).unwrap();
    let mut filedata: Vec<u8> = Vec::new();
    let f = z_jar.by_name("META-INF/MANIFEST.MF");
    if f.is_err() {
        error!(f.err().unwrap());
        exit(1)
    }
    f.unwrap().read_to_end(&mut filedata).unwrap();
    let data = String::from_utf8(filedata).unwrap();
    let mut class: String = "".to_string();
    for l in data.split("\n") {
        if l.starts_with("Main-Class:") {
            let l = l.replace("Main-Class: ", "");
            log!(format!("Main class found! The class is {}", l));
            class = l;
        }
    }
    if class == "" {
        error!("Did not find the main class entry in your manifest.mf! Exiting...");
        exit(1);
    }
    class
}

/// Decrypts an encrypted class and returns byte vector
/// # Arguments
/// * `class_data` - Data of the class to be decrypted
fn decrypt_class_bytes(class_data: &mut Vec<u8>) {
    let key = Key::from_slice(config().license.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(NONCE.as_bytes());
    let a = cipher.decrypt_in_place(nonce, b"", class_data);
    if a.is_err() {
        error!("Error encountered when decrypting a class!");
        println!("{}", a.unwrap_err().to_string());
        exit(1);
    }
    a.unwrap();
}

/// As the very long but descriptive name suggests, this function strips the name data from the stored class bytes.
/// NOTE: THIS DOES NOT CHECK IF THE CLASS IS A PRE-STRIPPED CLASS OR NOT. PLEASE USE CAREFULLY
///
/// # Arguments
/// * `class_bytes` - The class byte vector to strip the name data from
fn strip_name_data_from_class_bytes(class_bytes: &mut Vec<u8>) {
    let cuts = (class_bytes.get(0).unwrap().clone()+1) as usize;
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

/// Checks if the jar that has to be loaded exists and if so, returns it.
fn get_jar() -> File {
    let name = config().input_jar.clone();
    if !Path::exists(Path::new(&name)) {
        error!("Input jar not found!");
        exit(1);
    }
    let jar = File::open(name);
    if jar.is_err() {
        error!(format!("{}", jar.unwrap_err()));
        exit(1);
    }
    jar.unwrap()
}

/// Tests if the key provided is valid by using a test file that would have been packed during the encryption process.
/// If it is not, then this function will exit the program.
///
/// # Arguments
/// * `jar` - The jar that needs to be tested.
fn test_jar(jar: &File) {
    test_key();
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

    let key = Key::from_slice(config().license.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(NONCE.as_bytes());
    let a = cipher.decrypt_in_place(nonce, b"", &mut d);
    if a.is_err() {
        error!("The license is invalid! Exiting...");
        exit(1);
    }
    a.unwrap();

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
    log!("The key is valid!")
}

/// Test the key provided
fn test_key() {
    let key = config().license.clone();
    if key.len() != 32 {
        error!(format!("Your key looks invalid! Are you sure you are using the right key?"));
        exit(1);
    }
}

#[actix_web::main]
pub async fn webserver() {
    HttpServer::new(|| {
        App::new()
            .service(handle_page)
            .service(handle_assets)
            .service(handle_api)
    })
        .bind(("127.0.0.1", 8080)).unwrap()
        .run()
        .await.unwrap();
}
#[get("/")]
async fn handle_page() -> impl Responder {
    let index = include_bytes!("webserver/index.html");
    HttpResponse::Ok().status(StatusCode::OK).body(index.as_slice())
}

#[get("/assets/{filename}")]
async fn handle_assets(req: HttpRequest) -> impl Responder {
    match req.match_info().get("filename").unwrap() {
        "glow.css" => {
            let css = include_bytes!("webserver/glow.css");
            HttpResponse::Ok().status(StatusCode::OK).body(css.as_slice())
        },
        "Skidpacker.png" => {
            let png = include_bytes!("webserver/Skidpacker.png");
            HttpResponse::Ok().status(StatusCode::OK).body(png.as_slice())
        },
        "main.js" => {
            let js = include_bytes!("webserver/main.js");
            HttpResponse::Ok().status(StatusCode::OK).body(js.as_slice())
        }
        _ => {
            HttpResponse::Ok().status(StatusCode::NOT_FOUND).body("Not found!")
        }
    }
}

#[get("/api/data")]
async fn handle_api() -> impl Responder {
    let d = *CLASS_COUNT.get().unwrap().read().unwrap();
    if d == 0 {
        let data = json!({"accepted":"Loading...","rejected":"Loading...","name":"Loading..."});
        return HttpResponse::Ok().status(StatusCode::OK).json(data);
    }
    let data = json!({"accepted": d, "name": config().input_jar});
    HttpResponse::Ok().status(StatusCode::OK).json(data)
}

fn increment_web_class_count() {
    *CLASS_COUNT.get().unwrap().write().unwrap() += 1;
}







/// Get the config
fn config() -> &'static Config {
    CONFIG.get().unwrap()
}


