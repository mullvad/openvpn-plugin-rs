/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version logs using the `error!` macro of the log crate. Compile without the `log`
/// feature to make it print to stderr.
#[cfg(feature = "log")]
pub fn log_error<E: ::std::error::Error>(error: E) {
    error!("{}", format_error(error));
}

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version only prints to stderr. Build the crate with the `log` feature to log using
/// the `error!` macro.
#[cfg(not(feature = "log"))]
pub fn log_error<E: ::std::error::Error>(error: E) {
    use std::io::{self, Write};
    let error_msg = format!("{}\n", format_error(error));

    let mut stderr = io::stderr();
    let _ = stderr.write_all(error_msg.as_bytes());
    let _ = stderr.flush();
}

fn format_error<E: ::std::error::Error>(error: E) -> String {
    let mut error_string = format!("Error: {}", error);
    let mut error_iter = error.cause();
    while let Some(e) = error_iter {
        error_string.push_str(&format!("\nCaused by: {}", e));
        error_iter = e.cause();
    }
    error_string
}
