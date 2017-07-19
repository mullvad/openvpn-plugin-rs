// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Constants for OpenVPN. Taken from include/openvpn-plugin.h in the OpenVPN repository:
//! https://github.com/OpenVPN/openvpn/blob/master/include/openvpn-plugin.h.in

use std::os::raw::{c_char, c_int, c_uint, c_void};

/// Struct sent to `openvpn_plugin_open_v3` containing input values.
#[repr(C)]
pub struct openvpn_plugin_args_open_in {
    type_mask: c_int,
    pub argv: *const *const c_char,
    pub envp: *const *const c_char,
    callbacks: *const c_void,
    ssl_api: ovpnSSLAPI,
    ovpn_version: *const c_char,
    ovpn_version_major: c_uint,
    ovpn_version_minor: c_uint,
    ovpn_version_patch: *const c_char,
}

#[allow(dead_code)]
#[repr(C)]
enum ovpnSSLAPI {
    None,
    OpenSsl,
    MbedTls,
}

/// Struct used for returning values from `openvpn_plugin_open_v3` to OpenVPN.
#[repr(C)]
pub struct openvpn_plugin_args_open_return {
    pub type_mask: c_int,
    pub handle: *const c_void,
    return_list: *const c_void,
}

/// Struct sent to `openvpn_plugin_func_v3` containing input values.
#[repr(C)]
pub struct openvpn_plugin_args_func_in {
    pub event_type: c_int,
    pub argv: *const *const c_char,
    pub envp: *const *const c_char,
    pub handle: *const c_void,
    per_client_context: *const c_void,
    current_cert_depth: c_int,
    current_cert: *const c_void,
}

/// Struct used for returning values from `openvpn_plugin_func_v3` to OpenVPN.
#[repr(C)]
pub struct openvpn_plugin_args_func_return {
    return_list: *const c_void,
}
