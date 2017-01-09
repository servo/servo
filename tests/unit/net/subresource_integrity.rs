/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::subresource_integrity::{SriEntry, get_prioritized_hash_function, get_strongest_metadata};
use net::subresource_integrity::{is_response_integrity_valid, parsed_metadata};
use net_traits::response::{Response, ResponseBody};
use servo_url::ServoUrl;

#[test]
fn test_get_prioritized_hash_function() {
    let mut algorithm = get_prioritized_hash_function("sha256", "sha256");
    assert_eq!(algorithm, None);

    algorithm = get_prioritized_hash_function("sha256", "sha384");
    assert_eq!(algorithm.unwrap(), "sha384");

    algorithm = get_prioritized_hash_function("sha384", "sha512");
    assert_eq!(algorithm.unwrap(), "sha512");
}

#[test]
fn test_parsed_metadata_without_options() {
    let integrity_metadata = "sha384-Hash1";
    let ref parsed_metadata: SriEntry = parsed_metadata(integrity_metadata)[0];

    assert_eq!(parsed_metadata.alg, "sha384");
    assert_eq!(parsed_metadata.val, "Hash1");
    assert!(parsed_metadata.opt.is_none());
}

#[test]
fn test_parsed_metadata_with_options() {
    let integrity_metadata = "sha384-Hash1?opt=23";
    let ref parsed_metadata: SriEntry = parsed_metadata(integrity_metadata)[0];

    assert_eq!(parsed_metadata.alg, "sha384");
    assert_eq!(parsed_metadata.val, "Hash1");
    assert!(parsed_metadata.opt.is_some());
}

#[test]
fn test_parsed_metadata_with_malformed_integrity() {
    let integrity_metadata = "Not a valid integrity";
    let ref parsed_metadata_list: Vec<SriEntry> = parsed_metadata(integrity_metadata);

    assert!(parsed_metadata_list.is_empty());
}

#[test]
fn test_get_strongest_metadata_two_same_algorithm() {
    let integrity_metadata = "sha512-Hash1 sha512-Hash2?opt=23";
    let parsed_metadata_list: Vec<SriEntry> = parsed_metadata(integrity_metadata);

    let strong_metadata: Vec<SriEntry> = get_strongest_metadata(parsed_metadata_list);
    assert_eq!(strong_metadata.len(), 2);
    assert_eq!(strong_metadata[0].alg, strong_metadata[1].alg);
}

#[test]
fn test_get_strongest_metadata_different_algorithm() {
    let integrity_metadata = "sha256-Hash0 sha384-Hash1 sha512-Hash2?opt=23";
    let parsed_metadata_list: Vec<SriEntry> = parsed_metadata(integrity_metadata);

    let strong_metadata: Vec<SriEntry> = get_strongest_metadata(parsed_metadata_list);
    assert_eq!(strong_metadata.len(), 1);
    assert_eq!(strong_metadata[0].alg, "sha512");
}

#[test]
fn test_response_integrity_valid() {
    let url: ServoUrl = ServoUrl::parse("http://servo.org").unwrap();
    let response: Response = Response::new(url);

    let integrity_metadata = "sha384-H8BRh8j48O9oYatfu5AZzq6A9RINhZO5H16dQZngK7T62em8MUt1FLm52t+eX6xO";
    let response_body = "alert('Hello, world.');".to_owned().into_bytes();

    *response.body.lock().unwrap() = ResponseBody::Done(response_body);
    assert!(is_response_integrity_valid(integrity_metadata, &response));
}

#[test]
fn test_response_integrity_invalid() {
    let url: ServoUrl = ServoUrl::parse("http://servo.org").unwrap();
    let response: Response = Response::new(url);

    let integrity_metadata = "sha256-H8BRh8j48O9oYatfu5AZzq6A9RINhZO5H16dQZngK7T62em8MUt1FLm52t+eX6xO";
    let response_body = "alert('Hello, world.');".to_owned().into_bytes();

    *response.body.lock().unwrap() = ResponseBody::Done(response_body);
    assert!(!is_response_integrity_valid(integrity_metadata, &response));
}
