mod config;
mod web;

use jni::JNIEnv;
use jni::objects::{JClass, JObject, JString};
use lazy_static::lazy_static;
use crate::config::Config;
use std::sync::Mutex;

lazy_static! {
    pub static ref JNI_PTR: Mutex<usize> = Mutex::new(0);
    pub static ref CONFIG: Mutex<Config> = Mutex::new(Config::default());
}

/// The init function that has to be run to get the JNI pointer
/// # Arguments
/// * `env` - The JNI env pointer
/// * `_class` - Unused
/// * `configPath` - Path to the config file to load
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_dev_skidpacker_loader_jni_init(env: *mut u8, _class: JClass, configPath: JString) {
    *JNI_PTR.lock().unwrap() = env as usize;
    let cfg_path: String = get_jni_env().get_string(configPath).unwrap().into();
    let cfg: Config = Config::load(cfg_path.as_str());
    *CONFIG.lock().unwrap() = cfg;
}


/// Decrypts an encrypted class and returns byte vector
/// # Arguments
/// * `class_name` - Name of the class to be decrypted.
///
fn decrypt_class_bytes(class_name: &str) -> Vec<u8> {
    let class_bytes = get_class_bytes(class_name);
    /* Rest of decrypt process here */
    Vec::new() //REPLACE LATER
}


/// Load the encrypted class by name
/// # Arguments
/// * `class_name` - The name of the class to be loaded
fn load_encrypted_class(class_name: &str) -> Result<bool, String> {
    let decrypted_class_bytes = decrypt_class_bytes(class_name);
    /* Rest of the load code here */
    get_jni_env().define_unnamed_class(JObject::null(), &*decrypted_class_bytes).expect("TODO: panic message");
    Ok(true)
}

/// Get the bytes of a class by name
/// # Arguments
/// * `class_name` - Name of the class to get the bytes of
fn get_class_bytes(class_name: &str) -> Vec<u8> {
    Vec::new()
}

/// Get the JNI env
/// The value from the JNI PTR is used as a raw pointer
fn get_jni_env() -> JNIEnv<'static> {
    let ptr = *JNI_PTR.lock().unwrap();
    unsafe { JNIEnv::from_raw(ptr as *mut _).unwrap() }
}

/// Reads from the config and decodes the license into a byte vector
/// Returns a Vector of bytes
fn decode_license() -> Vec<u8> {
    let license: String = (*CONFIG.lock().unwrap().license).to_owned();
    base64::decode(license).unwrap()
}
