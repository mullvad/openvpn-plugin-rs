// Copyright 2023 Mullvad VPN AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_int;

/// Contains functions for parsing C formatted data from OpenVPN.
pub mod parse;

/// Rust representations of the C structs sent in and expected back by OpenVPN.
mod structs;
pub use self::structs::*;

// Return values. Returned from the plugin to OpenVPN to indicate success or failure. Can also
// Accept (success) or decline (error) operations, such as incoming client connection attempts.
pub const OPENVPN_PLUGIN_FUNC_SUCCESS: c_int = 0;
pub const OPENVPN_PLUGIN_FUNC_ERROR: c_int = 1;
pub const OPENVPN_PLUGIN_FUNC_DEFERRED: c_int = 2;
