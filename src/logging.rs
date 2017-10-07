/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version logs using the `error!` macro of the log crate. Compile without the `log`
/// feature to make it print to stderr.
#[cfg(feature = "log")]
mod implementation {
    use std::any::Any;

    pub fn log_error<E: ::std::error::Error>(error: E) {
        error!("{}", super::format_error(error));
    }

    pub fn log_panic(source: &str, panic_payload: Box<Any + Send + 'static>) {
        let panic_msg = panic_payload.downcast_ref::<&str>().unwrap_or(&"");
        error!("Panic in the {} callback: {:?}", source, panic_msg);
    }
}

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. This version only prints to stderr. Build the crate with the `log` feature to log using
/// the `error!` macro.
#[cfg(not(feature = "log"))]
mod implementation {
    use std::any::Any;
    use std::io::{self, Write};

    pub fn log_error<E: ::std::error::Error>(error: E) {
        let error_msg = format!("{}\n", super::format_error(error));

        let mut stderr = io::stderr();
        let _ = stderr.write_all(error_msg.as_bytes());
        let _ = stderr.flush();
    }

    pub fn log_panic(source: &str, panic_payload: Box<Any + Send + 'static>) {
        let panic_msg = panic_payload.downcast_ref::<&str>().unwrap_or(&"");
        let msg = format!("Panic in the {} callback: {:?}", source, panic_msg);

        let mut stderr = io::stderr();
        let _ = stderr.write_all(msg.as_bytes());
        let _ = stderr.flush();
    }
}

pub use self::implementation::*;

fn format_error<E: ::std::error::Error>(error: E) -> String {
    let mut error_string = format!("Error: {}", error);
    let mut error_iter = error.cause();
    while let Some(e) = error_iter {
        error_string.push_str(&format!("\nCaused by: {}", e));
        error_iter = e.cause();
    }
    error_string
}
