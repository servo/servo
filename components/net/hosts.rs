/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::LazyLock;

use parking_lot::Mutex;
use servo_config::opts;

static HOST_TABLE: LazyLock<Mutex<Option<HashMap<String, IpAddr>>>> =
    LazyLock::new(|| Mutex::new(create_host_table()));

fn create_host_table() -> Option<HashMap<String, IpAddr>> {
    let path = env::var_os("HOST_FILE")
        .map(PathBuf::from)
        .or_else(|| opts::get().host_file.clone())?;

    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    let mut lines = String::new();
    reader.read_to_string(&mut lines).ok()?;

    Some(parse_hostsfile(&lines))
}

#[cfg_attr(not(feature = "test-util"), expect(dead_code))]
pub fn replace_host_table(table: HashMap<String, IpAddr>) {
    *HOST_TABLE.lock() = Some(table);
}

pub fn parse_hostsfile(hostsfile_content: &str) -> HashMap<String, IpAddr> {
    hostsfile_content
        .lines()
        .filter_map(|line| {
            let mut iter = line.split('#').next().unwrap().split_whitespace();
            Some((iter.next()?.parse().ok()?, iter))
        })
        .flat_map(|(ip, hosts)| {
            hosts
                .filter(|host| {
                    let invalid = [
                        '\0', '\t', '\n', '\r', ' ', '#', '%', '/', ':', '?', '@', '[', '\\', ']',
                    ];
                    host.parse::<Ipv4Addr>().is_err() && !host.contains(&invalid[..])
                })
                .map(move |host| (host.to_owned(), ip))
        })
        .collect()
}

fn lookup_host_replacement(table: &HashMap<String, IpAddr>, host: &str) -> Option<IpAddr> {
    table.get(host).copied().or_else(|| {
        // <https://www.w3.org/TR/CSP/#grammardef-host-part>
        // host-part   = "*" / [ "*." ] 1*host-char *( "." 1*host-char ) [ "." ]
        host.strip_suffix('.')
            .filter(|host_without_trailing_dot| !host_without_trailing_dot.is_empty())
            .and_then(|host_without_trailing_dot| table.get(host_without_trailing_dot).copied())
    })
}

pub fn replace_host(host: &str) -> Cow<'_, str> {
    HOST_TABLE
        .lock()
        .as_ref()
        .and_then(|table| lookup_host_replacement(table, host))
        .map_or(host.into(), |replaced_host| {
            replaced_host.to_string().into()
        })
}
