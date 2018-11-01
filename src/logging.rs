use std::any::Any;

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version logs using the `error!` macro of the log crate. Compile without the `log`
/// feature to make it print to stderr.
#[cfg(feature = "log")]
macro_rules! log_error {
    ($error:expr) => {
        error!("{}", logging::format_error(&$error));
    };
}

#[cfg(feature = "log")]
macro_rules! log_panic {
    ($source:expr, $panic_payload:expr) => {
        error!("{}", logging::format_panic($source, $panic_payload));
    };
}

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version only prints to stderr. Build the crate with the `log` feature to log using
/// the `error!` macro.
#[cfg(not(feature = "log"))]
macro_rules! log_error {
    ($error:expr) => {{
        logging::try_write_stderr(&logging::format_error(&$error));
    }};
}

#[cfg(not(feature = "log"))]
macro_rules! log_panic {
    ($source:expr, $panic_payload:expr) => {{
        logging::try_write_stderr(&logging::format_panic($source, $panic_payload));
    }};
}

#[cfg(not(feature = "log"))]
pub fn try_write_stderr(msg: &str) {
    use std::io::{self, Write};
    let mut stderr = io::stderr();
    let _ = writeln!(stderr, "{}", msg);
}


pub fn format_error<E: ::std::error::Error>(error: &E) -> String {
    let mut error_string = format!("Error: {}", error);
    let mut error_iter = error.cause();
    while let Some(e) = error_iter {
        error_string.push_str(&format!("\nCaused by: {}", e));
        error_iter = e.cause();
    }
    error_string
}

pub fn format_panic(source: &str, panic_payload: &Box<Any + Send + 'static>) -> String {
    static NO_MSG: &'static str = "No panic message";
    let panic_msg = panic_payload.downcast_ref::<&str>().unwrap_or(&NO_MSG);
    format!("Panic in the {} callback: {:?}", source, panic_msg)
}
