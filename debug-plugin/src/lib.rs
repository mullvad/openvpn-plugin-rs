// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This debug/example OpenVPN plugin listens for almost all events and prints the arguments
//! for each event callback and returns success in every case.

#[macro_use]
extern crate openvpn_plugin;

use openvpn_plugin::types::{OpenVpnPluginEvent, EventResult};
use std::collections::HashMap;
use std::ffi::CString;

/// The list of OpenVPN events we register for. The list contains all possible events, but some of
/// them are commented out since they work slightly different, and will not work with the simple
/// log-and-return-success implementation we have here.
pub static INTERESTING_EVENTS: &[OpenVpnPluginEvent] = &[
    OpenVpnPluginEvent::Up,
    OpenVpnPluginEvent::Down,
    OpenVpnPluginEvent::RouteUp,
    OpenVpnPluginEvent::IpChange,
    // OpenVpnPluginEvent::TlsVerify,
    // OpenVpnPluginEvent::AuthUserPassVerify,
    OpenVpnPluginEvent::ClientConnect,
    OpenVpnPluginEvent::ClientDisconnect,
    OpenVpnPluginEvent::LearnAddress,
    OpenVpnPluginEvent::ClientConnectV2,
    OpenVpnPluginEvent::TlsFinal,
    OpenVpnPluginEvent::EnablePf,
    OpenVpnPluginEvent::RoutePredown,
    // OpenVpnPluginEvent::N,
];

openvpn_plugin!(::openvpn_open, ::lol::openvpn_close, ::openvpn_event, ());

fn openvpn_open(
    args: &[CString],
    env: &HashMap<CString, CString>,
) -> Result<(Vec<OpenVpnPluginEvent>, ()), ::std::io::Error> {
    println!(
        "DEBUG-PLUGIN: open called:\n\targs: {:?}\n\tenv: {:?}",
        args,
        env
    );
    Ok((INTERESTING_EVENTS.to_vec(), ()))
}

mod lol {
    pub fn openvpn_close(_handle: ()) {
        println!("DEBUG-PLUGIN: close called")
    }
}

fn openvpn_event(
    event: OpenVpnPluginEvent,
    args: &[CString],
    env: &HashMap<CString, CString>,
    _handle: &mut (),
) -> Result<EventResult, ::std::io::Error> {
    println!(
        "DEBUG-PLUGIN: event called:\n\tevent: {:?}\n\targs: {:?}\n\tenv: {:?}",
        event,
        args,
        env
    );
    Ok(EventResult::Success)
}
