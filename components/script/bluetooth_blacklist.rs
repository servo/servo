/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::string::String;

const EXCLUDE_READS: &'static str = "exclude-reads";
const EXCLUDE_WRITES: &'static str = "exclude-writes";
const FILE_PATH: &'static str = "./resources/gatt_blacklist.txt";
const VALID_UUID_REGEX: &'static str = "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

thread_local!(pub static BLUETOOTH_BLACKLIST: RefCell<BluetoothBlacklist> =
              RefCell::new(BluetoothBlacklist(parse_blacklist())));

macro_rules! return_if_blacklisted {
    ($uuid:expr, $exclude_fn:ident) => {
        if BLUETOOTH_BLACKLIST.with(|blist| {
            blist.borrow().$exclude_fn($uuid.as_ref())
        }) {
            return Err(Security);
        }
    }
}

pub struct BluetoothBlacklist(Option<HashMap<String, ExcludeType>>);

#[derive(Eq, PartialEq)]
pub enum ExcludeType {
    Exclude, // Read and Write
    ExcludeReads,
    ExcludeWrites,
}

impl BluetoothBlacklist {

    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted
    pub fn is_blacklisted(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&ExcludeType::Exclude)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted-for-reads
    pub fn is_blacklisted_for_reads(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&ExcludeType::Exclude) ||
                                                              et.eq(&ExcludeType::ExcludeReads)),
            None => false,
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#blacklisted-for-writes
    pub fn is_blacklisted_for_writes(&self, uuid: &str) -> bool {
        match self.0 {
            Some(ref map) => map.get(uuid).map_or(false, |et| et.eq(&ExcludeType::Exclude) ||
                                                              et.eq(&ExcludeType::ExcludeWrites)),
            None => false,
        }
    }
}

// https://webbluetoothcg.github.io/web-bluetooth/#parsing-the-blacklist
fn parse_blacklist() -> Option<HashMap<String, ExcludeType>> {
    // Step 1 missing, currently we parse ./resources/gatt_blacklist.txt.
    let file = match File::open(FILE_PATH) {
        Ok(file) => file,
        Err(_) => return None,
    };
    let content = BufReader::new(file);
    let valid_uuid_regex = Regex::new(VALID_UUID_REGEX).unwrap();
    // Step 3
    let mut result = HashMap::new();
    // Step 2 and 4
    for line in content.lines() {
        if let Ok(line) = line {
            // Step 4.1
            if !line.is_empty() && !line.starts_with('#') {
                let mut exclude_type = ExcludeType::Exclude;
                let mut words = line.split_whitespace();
                if let Some(uuid) = words.next() {
                    if valid_uuid_regex.is_match(uuid) {
                        match words.next() {
                            // Step 4.2 We already have an initialized exclude_type with ExcludeType::Exclude.
                            None => {},
                            // Step 4.3
                            Some(EXCLUDE_READS) => {
                                exclude_type = ExcludeType::ExcludeReads;
                            },
                            Some(EXCLUDE_WRITES) => {
                                exclude_type  = ExcludeType::ExcludeWrites;
                            },
                            // Step 4.4
                            _ => {
                                return None;
                            },
                        }
                    } else {
                        return None;
                    }
                    // Step 4.5
                    if result.contains_key(uuid) {
                        return None;
                    }
                    // Step 4.6
                    result.insert(uuid.to_string(), exclude_type);
                }
            }
        } else {
            return None;
        }
    }
    // Step 5
    return Some(result);
}
