/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use regex::Regex;
use servo_config::resource_files::read_resource_file;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufRead;
use std::string::String;

const BLOCKLIST_FILE: &'static str = "gatt_blocklist.txt";
const BLOCKLIST_FILE_NOT_FOUND: &'static str = "Could not find gatt_blocklist.txt file";
const EXCLUDE_READS: &'static str = "exclude-reads";
const EXCLUDE_WRITES: &'static str = "exclude-writes";
const VALID_UUID_REGEX: &'static str = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

thread_local!(pub static BLUETOOTH_BLOCKLIST: RefCell<BluetoothBlocklist> =
              RefCell::new(BluetoothBlocklist(parse_blocklist())));

pub fn uuid_is_blocklisted(uuid: &str, exclude_type: Blocklist) -> bool {
    BLUETOOTH_BLOCKLIST.with(|blist| {
        match exclude_type {
            Blocklist::All => {
                blist.borrow().is_blocklisted(uuid)
            },
            Blocklist::Reads => {
                blist.borrow().is_blocklisted_for_reads(uuid)
            }
            Blocklist::Writes => {
                blist.borrow().is_blocklisted_for_writes(uuid)
            }
        }
    })
}

pub struct BluetoothBlocklist(Option<HashMap<String, Blocklist>>);

#[derive(Eq, PartialEq)]
pub enum Blocklist {
    All, // Read and Write
    Reads,
    Writes,
}

impl BluetoothBlocklist {
    // https://webbluetoothcg.github.io/web-bluetooth/#blocklisted
    pub fn is_blocklisted(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blocklist::All)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blocklisted-for-reads
    pub fn is_blocklisted_for_reads(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blocklist::All) ||
                                                              et.eq(&Blocklist::Reads)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blocklisted-for-writes
    pub fn is_blocklisted_for_writes(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&Blocklist::All) ||
                                                              et.eq(&Blocklist::Writes)),
            None => false,
        }
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#parsing-the-blocklist
fn parse_blocklist() -> Option<HashMap<String, Blocklist>> {
    // Step 1 missing, currently we parse ./resources/gatt_blocklist.txt.
    let valid_uuid_regex = Regex::new(VALID_UUID_REGEX).unwrap();
    let content = read_resource_file(BLOCKLIST_FILE).expect(BLOCKLIST_FILE_NOT_FOUND);
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
        let mut exclude_type = Blocklist::All;
        let mut words = line.split_whitespace();
        let uuid = match words.next() {
            Some(uuid) => uuid,
            None => continue,
        };
        if !valid_uuid_regex.is_match(uuid) {
            return None;
        }
        match words.next() {
            // Step 4.2 We already have an initialized exclude_type variable with Blocklist::All.
            None => {},
            // Step 4.3
            Some(EXCLUDE_READS) => {
                exclude_type = Blocklist::Reads;
            },
            Some(EXCLUDE_WRITES) => {
                exclude_type  = Blocklist::Writes;
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
