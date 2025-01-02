/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::iter::Filter;
use std::str::Split;
use std::sync::MutexGuard;

use base64::Engine;
use generic_array::ArrayLength;
use net_traits::response::{Response, ResponseBody, ResponseType};
use sha2::{Digest, Sha256, Sha384, Sha512};

const SUPPORTED_ALGORITHM: &[&str] = &["sha256", "sha384", "sha512"];
pub type StaticCharVec = &'static [char];
/// A "space character" according to:
///
/// <https://html.spec.whatwg.org/multipage/#space-character>
pub static HTML_SPACE_CHARACTERS: StaticCharVec =
    &['\u{0020}', '\u{0009}', '\u{000a}', '\u{000c}', '\u{000d}'];
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
            opt,
        }
    }
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#parse-metadata>
pub fn parsed_metadata(integrity_metadata: &str) -> Vec<SriEntry> {
    // Step 1
    let mut result = vec![];

    // Step 3
    let tokens = split_html_space_chars(integrity_metadata);
    for token in tokens {
        let parsed_data: Vec<&str> = token.split('-').collect();

        if parsed_data.len() > 1 {
            let alg = parsed_data[0];

            if !SUPPORTED_ALGORITHM.contains(&alg) {
                continue;
            }

            let data: Vec<&str> = parsed_data[1].split('?').collect();
            let digest = data[0];

            let opt = if data.len() > 1 {
                Some(data[1].to_owned())
            } else {
                None
            };

            result.push(SriEntry::new(alg, digest, opt));
        }
    }

    result
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#getprioritizedhashfunction>
pub fn get_prioritized_hash_function(
    hash_func_left: &str,
    hash_func_right: &str,
) -> Option<String> {
    let left_priority = SUPPORTED_ALGORITHM
        .iter()
        .position(|s| *s == hash_func_left)
        .unwrap();
    let right_priority = SUPPORTED_ALGORITHM
        .iter()
        .position(|s| *s == hash_func_right)
        .unwrap();

    if left_priority == right_priority {
        return None;
    }
    if left_priority > right_priority {
        Some(hash_func_left.to_owned())
    } else {
        Some(hash_func_right.to_owned())
    }
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#get-the-strongest-metadata>
pub fn get_strongest_metadata(integrity_metadata_list: Vec<SriEntry>) -> Vec<SriEntry> {
    let mut result: Vec<SriEntry> = vec![integrity_metadata_list[0].clone()];
    let mut current_algorithm = result[0].alg.clone();

    for integrity_metadata in &integrity_metadata_list[1..] {
        let prioritized_hash =
            get_prioritized_hash_function(&integrity_metadata.alg, &current_algorithm);
        if prioritized_hash.is_none() {
            result.push(integrity_metadata.clone());
        } else if let Some(algorithm) = prioritized_hash {
            if algorithm != current_algorithm {
                result = vec![integrity_metadata.clone()];
                current_algorithm = algorithm;
            }
        }
    }

    result
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#apply-algorithm-to-response>
fn apply_algorithm_to_response<S: ArrayLength<u8>, D: Digest<OutputSize = S>>(
    body: MutexGuard<ResponseBody>,
    mut hasher: D,
) -> String {
    if let ResponseBody::Done(ref vec) = *body {
        hasher.update(vec);
        let response_digest = hasher.finalize(); //Now hash
        base64::engine::general_purpose::STANDARD.encode(&response_digest)
    } else {
        unreachable!("Tried to calculate digest of incomplete response body")
    }
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#is-response-eligible>
fn is_eligible_for_integrity_validation(response: &Response) -> bool {
    matches!(
        response.response_type,
        ResponseType::Basic | ResponseType::Default | ResponseType::Cors
    )
}

/// <https://w3c.github.io/webappsec-subresource-integrity/#does-response-match-metadatalist>
pub fn is_response_integrity_valid(integrity_metadata: &str, response: &Response) -> bool {
    let parsed_metadata_list: Vec<SriEntry> = parsed_metadata(integrity_metadata);

    // Step 2 & 4
    if parsed_metadata_list.is_empty() {
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

        let hashed = match &*algorithm {
            "sha256" => apply_algorithm_to_response(body, Sha256::new()),
            "sha384" => apply_algorithm_to_response(body, Sha384::new()),
            "sha512" => apply_algorithm_to_response(body, Sha512::new()),
            _ => continue,
        };

        if hashed == digest {
            return true;
        }
    }

    false
}

pub fn split_html_space_chars(s: &str) -> Filter<Split<'_, StaticCharVec>, fn(&&str) -> bool> {
    fn not_empty(&split: &&str) -> bool {
        !split.is_empty()
    }
    s.split(HTML_SPACE_CHARACTERS)
        .filter(not_empty as fn(&&str) -> bool)
}
