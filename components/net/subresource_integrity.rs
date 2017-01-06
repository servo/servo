/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::response::{Response, ResponseBody, ResponseType};
use openssl::crypto::hash::{hash, Type as MessageDigest};
use rustc_serialize::base64::{STANDARD, ToBase64};
use std::collections::HashMap;
use std::sync::MutexGuard;
lazy_static! {
    //Key is supported algorithm and value is priority
    static ref SUPPORTED_ALGORITHM: HashMap<&'static str, usize> = {
         let mut map = HashMap::new();
         map.insert("sha256", 1);
         map.insert("sha384", 2);
         map.insert("sha512", 3);
         map
    };
}

#[derive(Clone)]
pub struct SriEntry {
    pub alg: String,
    pub val: String,
    // TODO : Current version of spec does not define any option.
    // Can be refactored into appropriate datastructure when future
    // spec has more details.
    pub opt: Option<String>,
}

impl SriEntry {
    pub fn new(alg: &str, val: &str, opt: Option<String>) -> SriEntry {
        SriEntry {
            alg: alg.to_owned(),
            val: val.to_owned(),
            opt: opt,
        }
    }
}

/// https://w3c.github.io/webappsec-subresource-integrity/#parse-metadata
pub fn parsed_metadata(integrity_metadata: &str) -> Vec<SriEntry> {
    // Step 1
    let mut result = vec![];

    // Step 3
    let tokens: Vec<&str> = integrity_metadata.split(" ").collect();

    for token in tokens {
        let parsed_data: Vec<&str> = token.split("-").collect();

        if parsed_data.len() > 0 {
            let alg = parsed_data[0];

            if SUPPORTED_ALGORITHM.contains_key(alg) {
                let data: Vec<&str> = parsed_data[1].split("?").collect();
                let digest = data[0];

                let opt = if data.len() > 1 {
                    Some(data[1].to_owned())
                } else {
                    None
                };

                result.push(SriEntry::new(alg, digest, opt))
            }
        }
    }

    return result;
}

/// https://w3c.github.io/webappsec-subresource-integrity/#getprioritizedhashfunction
pub fn get_prioritized_hash_function(hash_func_left: &str, hash_func_right: &str) -> String {
    let left_priority = SUPPORTED_ALGORITHM.get(hash_func_left).unwrap();
    let right_priority = SUPPORTED_ALGORITHM.get(hash_func_right).unwrap();

    if left_priority == right_priority {
        return "".to_owned();
    }
    if left_priority > right_priority {
        hash_func_left.to_owned()
    } else {
        hash_func_right.to_owned()
    }

}

/// https://w3c.github.io/webappsec-subresource-integrity/#get-the-strongest-metadata
pub fn get_strongest_metadata(integrity_metadata_list: Vec<SriEntry>) -> Vec<SriEntry> {
    let mut result: Vec<SriEntry> = vec![integrity_metadata_list[0].clone()];
    let mut current_algorithm = result[0].alg.clone();

    for i in 1..integrity_metadata_list.len() {
        let intgerity_metadata = integrity_metadata_list[i].clone();

        let prioritized_hash = get_prioritized_hash_function(&intgerity_metadata.alg,
                                                                 &*current_algorithm);
        if prioritized_hash == "" {
            result.push(intgerity_metadata.clone());
        } else if prioritized_hash != *current_algorithm {
            result = vec![intgerity_metadata.clone()];
            current_algorithm = prioritized_hash;
        }
    }

    result
}

/// https://w3c.github.io/webappsec-subresource-integrity/#apply-algorithm-to-response
fn apply_algorithm_to_response(body: MutexGuard<ResponseBody>,
                               message_digest: MessageDigest)
                               -> String {
    if let ResponseBody::Done(ref vec) = *body {
        let response_digest = hash(message_digest, vec);
        response_digest.to_base64(STANDARD)
    } else {
        "".to_owned()
    }
}

/// https://w3c.github.io/webappsec-subresource-integrity/#is-response-eligible
fn is_eligible_for_integrity_validation(response: &Response) -> bool {
    match response.response_type {
        ResponseType::Basic | ResponseType::Default | ResponseType::Cors => true,
        _ => false,
    }
}

/// https://w3c.github.io/webappsec-subresource-integrity/#does-response-match-metadatalist
pub fn is_response_integrity_valid(integrity_metadata: &str, response: &Response) -> bool {
    let parsed_metadata_list: Vec<SriEntry> = parsed_metadata(integrity_metadata);

    // Step 2 & 4
    if parsed_metadata_list.len() == 0 {
        return true;
    }

    // Step 3
    if !is_eligible_for_integrity_validation(response) {
        return false;
    }

    // Step 5
    let metadata: Vec<SriEntry> = get_strongest_metadata(parsed_metadata_list);
    for item in metadata {
        let body = response.body.lock().unwrap();
        let algorithm = item.alg;
        let digest = item.val;

        let message_digest = match &*algorithm {
            "sha256" => MessageDigest::SHA256,
            "sha384" => MessageDigest::SHA384,
            "sha512" => MessageDigest::SHA512,
            _ => continue,
        };

        if apply_algorithm_to_response(body, message_digest) == digest {
            return true;
        }
    }

    false
}
