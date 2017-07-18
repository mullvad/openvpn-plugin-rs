// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! `openvpn-plugin` is a crate that makes it easy to write OpenVPN plugins in Rust.
//!
//! The crate contains two main things:
//!
//! * The `openvpn_plugin!` macro for generating the FFI interface OpenVPN will interact with
//! * The FFI and safe Rust types needed to communicate with OpenVPN.
//!
//! ## Usage
//!
//! Edit your `Cargo.toml` to depend on this crate and set the type of your crate to a `cdylib` in
//! order to make it compile to a shared library that OpenVPN will understand:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! openvpn-plugin = "0.1"
//! ```
//!
//! Import the crate, including macros, in your crate root (`lib.rs`):
//!
//! ```rust,ignore
//! #[macro_use] extern crate openvpn_plugin;
//! ```
//!
//! Also in your crate root (`lib.rs`) define your handle type, the three callback functions and
//! call the `openvpn_plugin!` macro to generate the corresponding FFI bindings.
//! More details on the handle and the callback functions can be found in the documentation for the
//! `openvpn_plugin!` macro.
//!
//! ```rust,ignore
//! pub struct Handle {
//!     // Fields needed for the plugin to keep state between callbacks
//! }
//!
//! fn openvpn_open(
//!     _args: &[CString],
//!     _env: &HashMap<CString, CString>,
//! ) -> Result<(Vec<openvpn_plugin::types::OpenVpnPluginEvent>, Handle, ()> {
//!     // Listen to only the `Up` event, which will be fired when a tunnel has been established.
//!     let events = vec![OpenVpnPluginEvent::Up];
//!     // Create the handle instance.
//!     let handle = Handle { /* ... */ };
//!     Ok((events, handle))
//! }
//!
//! pub fn openvpn_close(_handle: Handle) {
//!     println!("Plugin is closing down");
//! }
//!
//! fn openvpn_event(
//!     _event: openvpn_plugin::types::OpenVpnPluginEvent,
//!     _args: &[CString],
//!     _env: &HashMap<CString, CString>,
//!     _handle: &mut Handle,
//! ) -> Result<SuccessType, ()> {
//!     /* Process the event */
//!
//!     // If the processing worked fine and/or the request the callback represents should be
//!     // accepted, return `SuccessType::Success`. See docs on this enum for more info.
//!     Ok(SuccessType::Success)
//! }
//!
//! openvpn_plugin!(::openvpn_open, ::openvpn_close, ::openvpn_event, Handle);
//! ```
//!

#[cfg(feature = "serialize")]
extern crate serde;
#[cfg_attr(feature = "serialize", macro_use)]
#[cfg(feature = "serialize")]
extern crate serde_derive;

#[cfg_attr(feature = "log", macro_use)]
#[cfg(feature = "log")]
extern crate log;

/// FFI types and functions used by the plugin to convert between the types OpenVPN pass and expect
/// back and the Rust types the plugin will be exposed to.
///
/// Not intended for manual use. Is publicly exported since code generated by the `openvpn_plugin`
/// macro must access these types and functions.
pub mod ffi;

/// Rust types representing values and instructions from and to OpenVPN. Intended to be the safe
/// abstraction exposed to the plugins.
pub mod types;


/// The main part of this crate. The macro generates the public FFI functions that OpenVPN looks
/// for in a shared library:
///
/// * `openvpn_plugin_open_v3` - Will call `$open_fn`
/// * `openvpn_plugin_close_v1` - Will call `$close_fn`
/// * `openvpn_plugin_func_v3` - Will call `$event_fn`
///
/// This macro must be called in the crate root of the crate you wish to become an OpenVPN plugin.
/// That is because the FFI functions must be publicly exported from the shared library for OpenVPN
/// to find them.
///
/// See the top level library documentation and the included `debug-plugin` crate for examples on
/// how to use this macro.
///
///
/// ## `$open_fn` - The plugin load callback
///
/// Should be a function with the following signature:
///
/// ```rust,ignore
/// fn foo_open(
///     args: &[CString],
///     env: &HashMap<CString, CString>
/// ) -> Result<(Vec<types::OpenVpnPluginEvent>, $handle_ty), _>
/// ```
///
/// With `foo_open` substituted for the function name of your liking and `$handle_ty` substituted
/// with the handle type you pass.
///
/// This function will be called by OpenVPN when the plugin is loaded, just as OpenVPN starts.
///
/// This function has access to the arguments passed to the plugin and the initial
/// OpenVPN environment. If the plugin deems the open operation successful it should return a vector
/// with the events it wants to register for and the handle instance that the plugin can use to
/// keep state (See further down for more on the handle).
///
/// The type of the error returned from this function does not matter, as long as it implements
/// `std::error::Error`. Any error returned is being logged with `log_error()`, and then
/// `openvpn_plugin` returns `OPENVPN_PLUGIN_FUNC_ERROR` to OpenVPN, which indicates that the plugin
/// failed to load and OpenVPN will abort and exit.
///
/// The `openvpn_plugin::ffi::parse::{string_array_utf8, env_utf8}` functions can be used to try
/// to convert the arguments and environment into Rust `String`s.
///
///
/// ## `$close_fn` - The plugin unload callback
///
/// Should be a function with the following signature:
///
/// ```rust,ignore
/// fn foo_close(handle: $handle_ty)
/// ```
///
/// This function is called just before the plugin is unloaded, just before OpenVPN shuts down.
/// Here the plugin can do any cleaning up that is necessary. Since the handle is passed by value it
/// will be dropped when this function returns.
///
///
/// ## `$event_fn` - The event callback function
///
/// Should be a function with the following signature:
///
/// ```rust,ignore
/// fn foo_event(
///     event: types::OpenVpnPluginEvent,
///     args: &[CString],
///     env: &HashMap<CString, CString>,
///     handle: &mut $handle_ty,
/// ) -> Result<types::SuccessType, _>
/// ```
///
/// This function is being called by OpenVPN each time one of the events that `$open_fn` registered
/// for happens. This can for example be that a tunnel is established or that a client wants to
/// authenticate.
///
/// The first argument, `OpenVpnPluginEvent`, will tell which event that is happening.
///
/// The type of the error returned from this function does not matter, as long as it implements
/// `std::error::Error`. Any error returned is being logged with `log_error()`, and then
/// `openvpn_plugin` returns `OPENVPN_PLUGIN_FUNC_ERROR` to OpenVPN, which indicates different
/// things on different events. In the case of an authentication request or TLS key verification it
/// means that the request is denied and the connection is aborted.
///
/// ## `$handle_ty` - The handle type
///
/// The handle must be created and returned by the `$open_fn` function and will be kept for the
/// entire runtime of the plugin. The handle is passed to every subsequent callback and this is the
/// way that the plugin is supposed to keep state between each callback.
///
#[macro_export]
macro_rules! openvpn_plugin {
    ($open_fn:path, $close_fn:path, $event_fn:path, $handle_ty:ty) => {
        mod openvpn_plugin_ffi {
            use std::os::raw::{c_int, c_void};
            use $crate::types::{OpenVpnPluginEvent, SuccessType};
            use $crate::ffi::*;
            use $crate::ffi::structs::*;

            /// Called by OpenVPN when the plugin is first loaded on OpenVPN start.
            /// Used to register which events the plugin wants to listen to (`args.type_mask`). Can
            /// also set an arbitrary pointer inside `args.handle` that will then be passed to all
            /// subsequent calls to the plugin.
            ///
            /// Will parse the data from OpenVPN and call the function given as `$open_fn` to the
            /// `openvpn_plugin` macro.
            #[no_mangle]
            pub extern "C" fn openvpn_plugin_open_v3(_version: c_int,
                                                    args: *const openvpn_plugin_args_open_in,
                                                    retptr: *mut openvpn_plugin_args_open_return)
                                                    -> c_int {
                let parsed_args = unsafe { $crate::ffi::parse::string_array((*args).argv) }
                    .expect("Malformed args from OpenVPN");
                let parsed_env = unsafe { $crate::ffi::parse::env((*args).envp) }
                    .expect("Malformed env from OpenVPN");

                match $open_fn(&parsed_args, &parsed_env) {
                    Ok((events, handle)) => {
                        let type_mask = $crate::types::events_to_bitmask(&events);
                        let handle_ptr = Box::into_raw(Box::new(handle)) as *const c_void;
                        unsafe {
                            (*retptr).type_mask = type_mask;
                            (*retptr).handle = handle_ptr;
                        }
                        OPENVPN_PLUGIN_FUNC_SUCCESS
                    },
                    Err(e) => {
                        $crate::log_error(e);
                        OPENVPN_PLUGIN_FUNC_ERROR
                    },
                }
            }

            /// Called by OpenVPN when the plugin is unloaded, just before OpenVPN shuts down.
            /// Will call the function given as `$event_fn` to the `openvpn_plugin` macro.
            #[no_mangle]
            pub extern "C" fn openvpn_plugin_close_v1(handle: *const c_void) {
                // IMPORTANT: Bring the handle object back from a raw pointer. This will cause the
                // handle object to be properly deallocated when `$close_fn` returns.
                let handle = *unsafe { Box::from_raw(handle as *mut $handle_ty) };
                $close_fn(handle);
            }

            /// Called by OpenVPN for each `OPENVPN_PLUGIN_*` event that it registered for in
            /// the open function.
            ///
            /// Will parse the data from OpenVPN and call the function given as `$event_fn` to the
            /// `openvpn_plugin` macro.
            #[no_mangle]
            pub extern "C" fn openvpn_plugin_func_v3(_version: c_int,
                                                    args: *const openvpn_plugin_args_func_in,
                                                    _retptr: *const openvpn_plugin_args_func_return)
                                                    -> c_int {
                let event_type = unsafe { (*args).event_type };
                let event = OpenVpnPluginEvent::from_int(event_type)
                    .expect("Invalid event integer");
                let parsed_args = unsafe { $crate::ffi::parse::string_array((*args).argv) }
                    .expect("Malformed args from OpenVPN");
                let parsed_env = unsafe { $crate::ffi::parse::env((*args).envp) }
                    .expect("Malformed env from OpenVPN");

                let mut handle = unsafe { Box::from_raw((*args).handle as *mut $handle_ty) };

                let result: Result<SuccessType, _> =
                    $event_fn(event, &parsed_args, &parsed_env, &mut handle);

                // Forget the handle again so it is not deallocated when we return here.
                Box::into_raw(handle);

                match result {
                    Ok(SuccessType::Success) => OPENVPN_PLUGIN_FUNC_SUCCESS,
                    Ok(SuccessType::Deferred) => OPENVPN_PLUGIN_FUNC_DEFERRED,
                    Err(e) => {
                        $crate::log_error(e);
                        OPENVPN_PLUGIN_FUNC_ERROR
                    },
                }
            }
        }
        // Export the openvpn_plugin_* FFI functions in the top level scope
        pub use openvpn_plugin_ffi::*;
    }
}



/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` returns an
/// error. This version logs using the `error!` macro of the log crate. Compile without the `log`
/// feature to make it print to stderr.
#[cfg(feature = "log")]
pub fn log_error<E: ::std::error::Error>(error: E) {
    error!("{}", format_error(error));
}

/// Error logging method used by the FFI functions to log if `$open_fn` or `$event_fn` returns an
/// error. This version only prints to stdout. Build the crate with the `log` feature to log using
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
