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
//! openvpn-plugin = "x.y"
//! ```
//!
//! In your crate root (`lib.rs`) define your handle type, the three callback functions and
//! call the [`openvpn_plugin!`] macro to generate the corresponding FFI bindings.
//! More details on the handle and the callback functions can be found in the documentation for the
//! [`openvpn_plugin!`] macro.
//!
//! ```rust,no_run
//! #[macro_use]
//! extern crate openvpn_plugin;
//!
//! use std::collections::HashMap;
//! use std::ffi::CString;
//! use std::io::Error;
//! use openvpn_plugin::types::{EventResult, OpenVpnPluginEvent};
//!
//! pub struct Handle {
//!     // Fields needed for the plugin to keep state between callbacks
//! }
//!
//! fn openvpn_open(
//!     args: Vec<CString>,
//!     env: HashMap<CString, CString>,
//! ) -> Result<(Vec<OpenVpnPluginEvent>, Handle), Error> {
//!     // Listen to only the `Up` event, which will be fired when a tunnel has been established.
//!     let events = vec![OpenVpnPluginEvent::Up];
//!     // Create the handle instance.
//!     let handle = Handle { /* ... */ };
//!     Ok((events, handle))
//! }
//!
//! fn openvpn_close(handle: Handle) {
//!     println!("Plugin is closing down");
//! }
//!
//! fn openvpn_event(
//!     event: OpenVpnPluginEvent,
//!     args: Vec<CString>,
//!     env: HashMap<CString, CString>,
//!     handle: &mut Handle,
//! ) -> Result<EventResult, Error> {
//!     /* Process the event */
//!
//!     // If the processing worked fine and/or the request the callback represents should be
//!     // accepted, return EventResult::Success. See EventResult docs for more info.
//!     Ok(EventResult::Success)
//! }
//!
//! openvpn_plugin!(::openvpn_open, ::openvpn_close, ::openvpn_event, Handle);
//! # fn main() {}
//! ```
//!
//! ## Panic handling
//!
//! C cannot handle Rust panic unwinding and thus it is not good practice to let Rust panic when
//! called from C. Because of this all calls from this crate to the callbacks given to
//! [`openvpn_plugin!`] \(`$open_fn`, `$close_fn` and `$event_fn`) are wrapped by
//! [`catch_unwind`].
//!
//! If [`catch_unwind`] captures a panic it will log it and then return
//! [`OPENVPN_PLUGIN_FUNC_ERROR`] to OpenVPN.
//!
//! Note that this will only work for unwinding panics, not with `panic=abort`.
//!
//! ## Logging
//!
//! Any errors returned from the user defined callbacks or panics that happens anywhere in Rust is
//! logged by this crate before control is returned to OpenVPN. By default logging happens to
//! stderr. To activate logging with the `error!` macro in the `log` crate, build this crate with
// the `log` feature.
//! [`openvpn_plugin!`]: macro.openvpn_plugin.html
//! [`OPENVPN_PLUGIN_FUNC_ERROR`]: ffi/constant.OPENVPN_PLUGIN_FUNC_ERROR.html
//! [`catch_unwind`]: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", macro_use)]
extern crate serde;

#[cfg_attr(feature = "log", macro_use)]
#[cfg(feature = "log")]
extern crate log;

use types::{EventResult, OpenVpnPluginEvent};

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_int, c_void};
use std::panic;

/// FFI types and functions used by the plugin to convert between the types OpenVPN pass and expect
/// back and the Rust types the plugin will be exposed to.
///
/// Not intended for manual use. Is publicly exported since code generated by the `openvpn_plugin`
/// macro must access these types and functions.
pub mod ffi;

/// Rust types representing values and instructions from and to OpenVPN. Intended to be the safe
/// abstraction exposed to the plugins.
pub mod types;

/// Functions for logging errors that occur in plugins.
#[macro_use]
mod logging;


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
/// ```rust,no_run
/// # use openvpn_plugin::types::OpenVpnPluginEvent;
/// # use std::ffi::CString;
/// # use std::collections::HashMap;
/// # struct Handle {}
/// # struct Error {}
/// fn foo_open(
///     args: Vec<CString>,
///     env: HashMap<CString, CString>
/// ) -> Result<(Vec<OpenVpnPluginEvent>, Handle), Error> {
///     /// ...
/// #    unimplemented!();
/// }
/// # fn main() {}
/// ```
///
/// With `foo_open` substituted for a function name of your liking and `Handle` being the
/// `$handle_ty` handle type you pass.
///
/// The type of the error in the result from this function does not matter, as long as it implements
/// `std::error::Error`. Any error returned is logged and then [`OPENVPN_PLUGIN_FUNC_ERROR`]
/// is returned to OpenVPN, which indicates that the plugin failed to load and OpenVPN will abort
/// and exit.
///
/// This function will be called by OpenVPN when the plugin is loaded, just as OpenVPN starts.
///
/// This function has access to the arguments passed to the plugin and the initial
/// OpenVPN environment. If the plugin deems the open operation successful it should return a vector
/// with the events it wants to register for and the handle instance that the plugin can use to
/// keep state (See further down for more on the handle).
///
/// The `openvpn_plugin::ffi::parse::{string_array_utf8, env_utf8}` functions can be used to try
/// to convert the arguments and environment into Rust `String`s.
///
///
/// ## `$close_fn` - The plugin unload callback
///
/// Should be a function with the following signature:
///
/// ```rust,no_run
/// # struct Handle {}
/// fn foo_close(handle: Handle) {
///     /// ...
/// #    unimplemented!();
/// }
/// # fn main() {}
/// ```
///
/// With `foo_close` substituted for a function name of your liking and `Handle` being the
/// `$handle_ty` handle type you pass.
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
/// ```rust,no_run
/// # use openvpn_plugin::types::{EventResult, OpenVpnPluginEvent};
/// # use std::ffi::CString;
/// # use std::collections::HashMap;
/// # struct Handle {}
/// # struct Error {}
/// fn foo_event(
///     event: OpenVpnPluginEvent,
///     args: Vec<CString>,
///     env: HashMap<CString, CString>,
///     handle: &mut Handle,
/// ) -> Result<EventResult, Error> {
///     /// ...
/// #    unimplemented!();
/// }
/// # fn main() {}
/// ```
///
/// With `foo_event` substituted for a function name of your liking and `Handle` being the
/// `$handle_ty` handle type you pass.
///
/// The type of the error in the result from this function does not matter, as long as it implements
/// `std::error::Error`. Any error returned is logged and then [`OPENVPN_PLUGIN_FUNC_ERROR`]
/// is returned to OpenVPN. [`OPENVPN_PLUGIN_FUNC_ERROR`] indicates different things on different
/// events. In the case of an authentication request or TLS key verification it means that the
/// request is denied and the connection is aborted.
///
/// This function is being called by OpenVPN each time one of the events that `$open_fn` registered
/// for happens. This can for example be that a tunnel is established or that a client wants to
/// authenticate.
///
/// The first argument, [`OpenVpnPluginEvent`], will tell which event that is happening.
///
///
/// ## `$handle_ty` - The handle type
///
/// The handle must be created and returned by the `$open_fn` function and will be kept for the
/// entire runtime of the plugin. The handle is passed to every subsequent callback and this is the
/// way that the plugin is supposed to keep state between each callback.
///
/// The handle instance is being dropped upon return from the `$close_fn` function just as the
/// plugin is being unloaded.
///
/// [`OpenVpnPluginEvent`]: types/enum.OpenVpnPluginEvent.html
/// [`OPENVPN_PLUGIN_FUNC_ERROR`]: ffi/constant.OPENVPN_PLUGIN_FUNC_ERROR.html
#[macro_export]
macro_rules! openvpn_plugin {
    ($open_fn:path, $close_fn:path, $event_fn:path, $handle_ty:ty) => {
        /// Called by OpenVPN when the plugin is first loaded on OpenVPN start.
        /// Used to register which events the plugin wants to listen to (`args.type_mask`). Can
        /// also set an arbitrary pointer inside `args.handle` that will then be passed to all
        /// subsequent calls to the plugin.
        ///
        /// Will parse the data from OpenVPN and call the function given as `$open_fn` to the
        /// `openvpn_plugin` macro.
        #[no_mangle]
        pub unsafe extern "C" fn openvpn_plugin_open_v3(
            _version: ::std::os::raw::c_int,
            args: *const $crate::ffi::openvpn_plugin_args_open_in,
            retptr: *mut $crate::ffi::openvpn_plugin_args_open_return,
        ) -> ::std::os::raw::c_int {
            unsafe { $crate::openvpn_plugin_open::<$handle_ty, _, _>(args, retptr, $open_fn) }
        }

        /// Called by OpenVPN when the plugin is unloaded, just before OpenVPN shuts down.
        /// Will call the function given as `$event_fn` to the `openvpn_plugin` macro.
        #[no_mangle]
        pub unsafe extern "C" fn openvpn_plugin_close_v1(handle: *const ::std::os::raw::c_void) {
            unsafe { $crate::openvpn_plugin_close::<$handle_ty, _>(handle, $close_fn) }
        }

        /// Called by OpenVPN for each `OPENVPN_PLUGIN_*` event that it registered for in
        /// the open function.
        ///
        /// Will parse the data from OpenVPN and call the function given as `$event_fn` to the
        /// `openvpn_plugin` macro.
        #[no_mangle]
        pub unsafe extern "C" fn openvpn_plugin_func_v3(
            _version: ::std::os::raw::c_int,
            args: *const $crate::ffi::openvpn_plugin_args_func_in,
            _retptr: *const $crate::ffi::openvpn_plugin_args_func_return,
        ) -> ::std::os::raw::c_int {
            unsafe { $crate::openvpn_plugin_func::<$handle_ty, _, _>(args, $event_fn) }
        }
    };
}


/// Internal macro for matching on a result and either return the value inside the `Ok`, or in the
/// case of an `Err`, log it and early return [`OPENVPN_PLUGIN_FUNC_ERROR`].
///
/// [`OPENVPN_PLUGIN_FUNC_ERROR`]: ffi/constant.OPENVPN_PLUGIN_FUNC_ERROR.html
macro_rules! try_or_return_error {
    ($result:expr, $error_msg:expr) => {
        match $result {
            Ok(result) => result,
            Err(e) => {
                log_error!(Error::new($error_msg, e));
                return ffi::OPENVPN_PLUGIN_FUNC_ERROR;
            }
        };
    };
}


/// Internal helper function. This function should never be called manually, only by code generated
/// by the [`openvpn_plugin!`] macro.
///
/// [`openvpn_plugin!`]: macro.openvpn_plugin.html
#[doc(hidden)]
pub unsafe fn openvpn_plugin_open<H, E, F>(
    args: *const ffi::openvpn_plugin_args_open_in,
    retptr: *mut ffi::openvpn_plugin_args_open_return,
    open_fn: F,
) -> c_int
where
    E: ::std::error::Error,
    F: panic::RefUnwindSafe,
    F: Fn(Vec<CString>, HashMap<CString, CString>) -> Result<(Vec<OpenVpnPluginEvent>, H), E>,
{
    let parsed_args = try_or_return_error!(
        ffi::parse::string_array((*args).argv),
        "Malformed args from OpenVPN"
    );
    let parsed_env =
        try_or_return_error!(ffi::parse::env((*args).envp), "Malformed env from OpenVPN");

    match panic::catch_unwind(|| open_fn(parsed_args, parsed_env)) {
        Ok(Ok((events, handle))) => {
            (*retptr).type_mask = types::events_to_bitmask(&events);
            (*retptr).handle = Box::into_raw(Box::new(handle)) as *const c_void;
            ffi::OPENVPN_PLUGIN_FUNC_SUCCESS
        }
        Ok(Err(e)) => {
            log_error!(e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
        Err(e) => {
            log_panic!("plugin open", &e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
    }
}


/// Internal helper function. This function should never be called manually, only by code generated
/// by the [`openvpn_plugin!`] macro.
///
/// [`openvpn_plugin!`]: macro.openvpn_plugin.html
#[doc(hidden)]
pub unsafe fn openvpn_plugin_close<H, F>(handle: *const c_void, close_fn: F)
where
    H: panic::UnwindSafe,
    F: Fn(H) + panic::RefUnwindSafe,
{
    // IMPORTANT: Bring the handle object back from a raw pointer. This will cause the
    // handle object to be properly deallocated when `$close_fn` returns.
    let handle = *Box::from_raw(handle as *mut H);
    if let Err(e) = panic::catch_unwind(|| close_fn(handle)) {
        log_panic!("plugin close", &e);
    }
}


/// Internal helper function. This function should never be called manually, only by code generated
/// by the [`openvpn_plugin!`] macro.
///
/// [`openvpn_plugin!`]: macro.openvpn_plugin.html
#[doc(hidden)]
pub unsafe fn openvpn_plugin_func<H, E, F>(
    args: *const ffi::openvpn_plugin_args_func_in,
    event_fn: F,
) -> c_int
where
    E: ::std::error::Error,
    F: panic::RefUnwindSafe,
    F: Fn(OpenVpnPluginEvent, Vec<CString>, HashMap<CString, CString>, &mut H)
        -> Result<EventResult, E>,
{
    let event = try_or_return_error!(
        OpenVpnPluginEvent::from_int((*args).event_type),
        "Invalid event integer"
    );
    let parsed_args = try_or_return_error!(
        ffi::parse::string_array((*args).argv),
        "Malformed args from OpenVPN"
    );
    let parsed_env =
        try_or_return_error!(ffi::parse::env((*args).envp), "Malformed env from OpenVPN");

    let result = panic::catch_unwind(|| {
        let handle: &mut H = &mut *((*args).handle as *mut H);
        event_fn(event, parsed_args, parsed_env, handle)
    });

    match result {
        Ok(Ok(EventResult::Success)) => ffi::OPENVPN_PLUGIN_FUNC_SUCCESS,
        Ok(Ok(EventResult::Deferred)) => ffi::OPENVPN_PLUGIN_FUNC_DEFERRED,
        Ok(Ok(EventResult::Failure)) => ffi::OPENVPN_PLUGIN_FUNC_ERROR,
        Ok(Err(e)) => {
            log_error!(e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
        Err(e) => {
            log_panic!("plugin func", &e);
            ffi::OPENVPN_PLUGIN_FUNC_ERROR
        }
    }
}


/// Internal error type
#[derive(Debug)]
struct Error {
    msg: &'static str,
    cause: Box<::std::error::Error>,
}

impl Error {
    pub fn new<E>(msg: &'static str, cause: E) -> Error
    where
        E: ::std::error::Error + 'static,
    {
        Error {
            msg,
            cause: Box::new(cause) as Box<::std::error::Error>,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        use std::error::Error;
        self.description().fmt(f)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.msg
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        Some(self.cause.as_ref())
    }
}
