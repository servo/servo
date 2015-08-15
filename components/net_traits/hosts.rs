/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use url::Url;

use {IPV4_REGEX, IPV6_REGEX};

static mut HOST_TABLE: Option<*mut HashMap<String, String>> = None;


pub fn global_init() {
    //TODO: handle bad file path
    let path = match env::var("HOST_FILE") {
        Ok(host_file_path) => host_file_path,
        Err(_) => return,
    };

    let mut file = match File::open(&path) {
        Ok(f) => BufReader::new(f),
        Err(_) => return,
    };

    let mut lines = String::new();
    match file.read_to_string(&mut lines) {
        Ok(_) => (),
        Err(_) => return,
    };

    unsafe {
        let host_table = Box::into_raw(parse_hostsfile(&lines));
        HOST_TABLE = Some(host_table);
    }
}

pub fn parse_hostsfile(hostsfile_content: &str) -> Box<HashMap<String, String>> {
    let mut host_table = HashMap::new();
    let lines: Vec<&str> = hostsfile_content.split('\n').collect();

    for line in &lines {
        let ip_host: Vec<&str> = line.trim().split(|c: char| c == ' ' || c == '\t').collect();
        if ip_host.len() > 1 {
            if !IPV4_REGEX.is_match(ip_host[0]) && !IPV6_REGEX.is_match(ip_host[0]) { continue; }
            let address = ip_host[0].to_owned();

            for token in ip_host.iter().skip(1) {
                if token.as_bytes()[0] == b'#' {
                    break;
                }
                host_table.insert((*token).to_owned(), address.clone());
            }
        }
    }
    box host_table
}

pub fn replace_hosts(url: &Url) -> Url {
    unsafe {
        HOST_TABLE.map(|host_table| {
            host_replacement(host_table, url)
        }).unwrap_or_else(|| url.clone())
    }
}

pub fn host_replacement(host_table: *mut HashMap<String, String>,
                        url: &Url) -> Url {
    unsafe {
        url.domain().and_then(|domain|
                              (*host_table).get(domain).map(|ip| {
                                  let mut net_url = url.clone();
                                  *net_url.domain_mut().unwrap() = ip.clone();
                                  net_url
                              })).unwrap_or(url.clone())
    }
}
