/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use parse_hosts::HostsFile;
use servo_url::ServoUrl;
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
    let mut host_table = HashMap::new();

    for line in HostsFile::read_buffered(hostsfile_content.as_bytes()).lines() {
        if let Ok(ref line) = line {
            for host in line.hosts() {
                if let Some(ip) = line.ip() {
                    host_table.insert(host.to_owned(), ip);
                }
            }
        }
    }

    host_table
}

pub fn replace_host(host: &str) -> Cow<str> {
    HOST_TABLE.lock().unwrap().as_ref()
        .and_then(|table| table.get(host))
        .map_or(host.into(), |replaced_host| replaced_host.to_string().into())
}

pub fn replace_host_in_url(url: ServoUrl) -> ServoUrl {
    if let Some(table) = HOST_TABLE.lock().unwrap().as_ref() {
        host_replacement(table, url)
    } else {
        url
    }
}

pub fn host_replacement(host_table: &HashMap<String, IpAddr>, mut url: ServoUrl) -> ServoUrl {
    let replacement = url.domain().and_then(|domain| host_table.get(domain));
    if let Some(ip) = replacement {
        url.set_ip_host(*ip).unwrap();
    }
    url
}
