use std::{any::Any, error::Error};

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` return an
/// error. It logs to the error log level of the `log` crate if the `log` feature is enabled.
/// Otherwise it will print the error to stderr.
pub fn log_error(error: &impl Error) {
    let error_msg = format_error(error);
    #[cfg(feature = "log")]
    {
        log::error!("{}", error_msg);
    }
    #[cfg(not(feature = "log"))]
    {
        eprintln!("{}", error_msg);
    }
}

pub fn log_panic(source: &str, panic_payload: &Box<Any + Send + 'static>) {
    let panic_msg = panic_payload
        .downcast_ref::<&str>()
        .unwrap_or(&"No panic message");

    #[cfg(feature = "log")]
    {
        log::error!("Panic in the {} callback: {:?}", source, panic_msg);
    }
    #[cfg(not(feature = "log"))]
    {
        eprintln!("Panic in the {} callback: {:?}", source, panic_msg);
    }
}

fn format_error<E: ::std::error::Error>(error: &E) -> String {
    let mut error_string = format!("Error: {}", error);
    let mut error_iter = error.cause();
    while let Some(e) = error_iter {
        error_string.push_str(&format!("\nCaused by: {}", e));
        error_iter = e.cause();
    }
    error_string
}
