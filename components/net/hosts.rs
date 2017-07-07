/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parse_hosts::HostsFile;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::IpAddr;
use std::sync::Mutex;

lazy_static! {
    static ref HOST_TABLE: Mutex<Option<HashMap<String, IpAddr>>> = Mutex::new(create_host_table());
}

fn create_host_table() -> Option<HashMap<String, IpAddr>> {
    // TODO: handle bad file path
    let path = match env::var("HOST_FILE") {
        Ok(host_file_path) => host_file_path,
        Err(_) => return None,
    };

    let mut file = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(_) => return None,
    };

    let mut lines = String::new();
    match file.read_to_string(&mut lines) {
        Ok(_) => (),
        Err(_) => return None,
    };

    Some(parse_hostsfile(&lines))
}

pub fn replace_host_table(table: HashMap<String, IpAddr>) {
    *HOST_TABLE.lock().unwrap() = Some(table);
}

pub fn parse_hostsfile(hostsfile_content: &str) -> HashMap<String, IpAddr> {
    HostsFile::read_buffered(hostsfile_content.as_bytes())
        .pairs()
        .filter_map(Result::ok)
        .collect()
}

pub fn replace_host(host: &str) -> Cow<str> {
    HOST_TABLE.lock().unwrap().as_ref()
        .and_then(|table| table.get(host))
        .map_or(host.into(), |replaced_host| replaced_host.to_string().into())
}
