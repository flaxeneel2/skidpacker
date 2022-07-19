/// Print function for verbose output
/// # Arguments
/// * `msg` - The message to output
#[macro_export]
macro_rules! verbose {
    ($msg: expr) => {
        if args().verbose { white_ln!("DEBUG: {}", $msg) }
    };
}

/// Print function for warning output
/// # Arguments
/// * `msg` - The message to output
#[macro_export]
macro_rules! warn {
    ($msg: expr) => {
        yellow_ln!("WARN:  {}", $msg)
    };
}

/// Print function for error output
/// # Arguments
/// * `msg` - The message to output
#[macro_export]
macro_rules! error {
    ($msg: expr) => {
        red_ln!("ERROR: {}", $msg)
    };
}

/// Print function for just basic log output
/// # Arguments
/// * `msg` - The message to output
#[macro_export]
macro_rules! log {
    ($msg: expr) => {
        blue_ln!("LOG:   {}", $msg)
    };
}
