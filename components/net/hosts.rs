/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Mutex;

lazy_static! {
    static ref HOST_TABLE: Mutex<Option<HashMap<String, IpAddr>>> = Mutex::new(create_host_table());
}

fn create_host_table() -> Option<HashMap<String, IpAddr>> {
    let path = env::var_os("HOST_FILE")?;

    let file = File::open(&path).ok()?;
    let mut reader = BufReader::new(file);

    let mut lines = String::new();
    reader.read_to_string(&mut lines).ok()?;

    Some(parse_hostsfile(&lines))
}

pub fn replace_host_table(table: HashMap<String, IpAddr>) {
    *HOST_TABLE.lock().unwrap() = Some(table);
}

pub fn parse_hostsfile(hostsfile_content: &str) -> HashMap<String, IpAddr> {
    hostsfile_content.lines().filter_map(|line| {
        let mut iter = line.split('#').next().unwrap().split_whitespace();
        Some((iter.next()?.parse().ok()?, iter))
    }).flat_map(|(ip, hosts)| {
        hosts.filter(|host| {
            let invalid = ['\0', '\t', '\n', '\r', ' ', '#', '%', '/', ':', '?', '@', '[', '\\', ']'];
            host.parse::<Ipv4Addr>().is_err() && !host.contains(&invalid[..])
        }).map(move |host| {
            (host.to_owned(), ip)
        })
    }).collect()
}

pub fn replace_host(host: &str) -> Cow<str> {
    HOST_TABLE.lock().unwrap().as_ref()
        .and_then(|table| table.get(host))
        .map_or(host.into(), |replaced_host| replaced_host.to_string().into())
}
