/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::{Ipv4Addr, Ipv6Addr};
use url::Url;

lazy_static! {
    static ref HOST_TABLE: Option<HashMap<String, String>> = create_host_table();
}

fn create_host_table() -> Option<HashMap<String, String>> {
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

pub fn parse_hostsfile(hostsfile_content: &str) -> HashMap<String, String> {
    let mut host_table = HashMap::new();
    for line in hostsfile_content.split('\n') {
        let ip_host: Vec<&str> = line.trim().split(|c: char| c == ' ' || c == '\t').collect();
        if ip_host.len() > 1 {
            if ip_host[0].parse::<Ipv4Addr>().is_err() && ip_host[0].parse::<Ipv6Addr>().is_err() {
                continue
            }
            let address = ip_host[0].to_owned();

            for token in ip_host.iter().skip(1) {
                if token.as_bytes()[0] == b'#' {
                    break;
                }
                host_table.insert((*token).to_owned(), address.clone());
            }
        }
    }
    host_table
}

pub fn replace_hosts(url: &Url) -> Url {
    HOST_TABLE.as_ref().map_or_else(|| url.clone(), |host_table| {
        host_replacement(host_table, url)
    })
}

pub fn host_replacement(host_table: &HashMap<String, String>,
                        url: &Url) -> Url {
    url.domain().and_then(|domain|
                          host_table.get(domain).map(|ip| {
                              let mut net_url = url.clone();
                              *net_url.domain_mut().unwrap() = ip.clone();
                              net_url
                          })).unwrap_or(url.clone())
}
