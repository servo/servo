/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use servo_url::ServoUrl;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::IpAddr;

lazy_static! {
    static ref HOST_TABLE: Option<HashMap<String, IpAddr>> = create_host_table();
}

fn create_host_table() -> Option<HashMap<String, IpAddr>> {
    //TODO: handle bad file path
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

    return Some(parse_hostsfile(&lines));
}

pub fn parse_hostsfile(hostsfile_content: &str) -> HashMap<String, IpAddr> {
    let mut host_table = HashMap::new();
    for line in hostsfile_content.split('\n') {
        let ip_host: Vec<&str> = line.trim().split(|c: char| c == ' ' || c == '\t').collect();
        if ip_host.len() > 1 {
            if let Ok(address) = ip_host[0].parse::<IpAddr>() {
                for token in ip_host.iter().skip(1) {
                    if token.as_bytes()[0] == b'#' {
                        break;
                    }
                    host_table.insert((*token).to_owned(), address);
                }
            }
        }
    }
    host_table
}

pub fn replace_hosts(url: &ServoUrl) -> ServoUrl {
    HOST_TABLE.as_ref().map_or_else(|| url.clone(), |host_table| {
        host_replacement(host_table, url)
    })
}

pub fn host_replacement(host_table: &HashMap<String, IpAddr>,
                        url: &ServoUrl) -> ServoUrl {
    url.domain().and_then(|domain| host_table.get(domain).map(|ip| {
        let mut new_url = url.clone();
        new_url.set_ip_host(*ip).unwrap();
        new_url
    })).unwrap_or_else(|| url.clone())
}
