/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufRead;
use std::string::String;
use util::resource_files::read_resource_file;

const BLACKLIST_FILE: &'static str = "gatt_blacklist.txt";
const BLACKLIST_FILE_NOT_FOUND: &'static str = "Could not find gatt_blacklist.txt file";
const EXCLUDE_READS: &'static str = "exclude-reads";
const EXCLUDE_WRITES: &'static str = "exclude-writes";
const VALID_UUID_REGEX: &'static str = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

thread_local!(pub static BLUETOOTH_BLACKLIST: RefCell<BluetoothBlacklist> =
              RefCell::new(BluetoothBlacklist(parse_blacklist())));

pub fn uuid_is_blacklisted(uuid: &str, exclude_type: Blacklist) -> bool {
    BLUETOOTH_BLACKLIST.with(|blist| {
        match exclude_type {
            Blacklist::All => {
                blist.borrow().is_blacklisted(uuid)
            },
            Blacklist::Reads => {
                blist.borrow().is_blacklisted_for_reads(uuid)
            }
            Blacklist::Writes => {
                blist.borrow().is_blacklisted_for_writes(uuid)
            }
        }
    })
}

pub struct BluetoothBlacklist(Option<HashMap<String, Blacklist>>);

#[derive(Eq, PartialEq)]
pub enum Blacklist {
    All, // Read and Write
    Reads,
    Writes,
}

impl BluetoothBlacklist {
    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted
    pub fn is_blacklisted(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blacklist::All)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted-for-reads
    pub fn is_blacklisted_for_reads(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blacklist::All) ||
                                                              et.eq(&Blacklist::Reads)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted-for-writes
    pub fn is_blacklisted_for_writes(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blacklist::All) ||
                                                              et.eq(&Blacklist::Writes)),
            None => false,
        }
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#parsing-the-blacklist
fn parse_blacklist() -> Option<HashMap<String, Blacklist>> {
    // Step 1 missing, currently we parse ./resources/gatt_blacklist.txt.
    let valid_uuid_regex = Regex::new(VALID_UUID_REGEX).unwrap();
    let content = read_resource_file(BLACKLIST_FILE).expect(BLACKLIST_FILE_NOT_FOUND);
    // Step 3
    let mut result = HashMap::new();
    // Step 2 and 4
    for line in content.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => return None,
        };
        // Step 4.1
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut exclude_type = Blacklist::All;
        let mut words = line.split_whitespace();
        let uuid = match words.next() {
            Some(uuid) => uuid,
            None => continue,
        };
        if !valid_uuid_regex.is_match(uuid) {
            return None;
        }
        match words.next() {
            // Step 4.2 We already have an initialized exclude_type variable with Blacklist::All.
            None => {},
            // Step 4.3
            Some(EXCLUDE_READS) => {
                exclude_type = Blacklist::Reads;
            },
            Some(EXCLUDE_WRITES) => {
                exclude_type  = Blacklist::Writes;
            },
            // Step 4.4
            _ => {
                return None;
            },
        }
        // Step 4.5
        if result.contains_key(uuid) {
            return None;
        }
        // Step 4.6
        result.insert(uuid.to_string(), exclude_type);
    }
    // Step 5
    return Some(result);
}
