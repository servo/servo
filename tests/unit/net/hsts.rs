/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::hsts::HSTSList;
use net::hsts::HSTSEntry;
use net_traits::IncludeSubdomains;
use net::hsts::{secure_url, preload_hsts_domains};
use net::resource_task::ResourceManager;
use ipc_channel::ipc;
use std::sync::mpsc::channel;
use url::Url;
use time;

#[test]
fn test_add_hsts_entry_to_resource_manager_adds_an_hsts_entry() {
    let list = HSTSList {
        entries: Vec::new()
    };

    let (tx, _) = ipc::channel().unwrap();
    let mut manager = ResourceManager::new("".to_owned(), tx, list, None);

    let entry = HSTSEntry::new(
        "mozilla.org".to_string(), IncludeSubdomains::NotIncluded, None
    );

    assert!(!manager.is_host_sts("mozilla.org"));

    manager.add_hsts_entry(entry.unwrap());

    assert!(manager.is_host_sts("mozilla.org"))
}

#[test]
fn test_hsts_entry_is_not_expired_when_it_has_no_timestamp() {
    let entry = HSTSEntry {
        host: "mozilla.org".to_string(),
        include_subdomains: false,
        max_age: Some(20),
        timestamp: None
    };

    assert!(!entry.is_expired());
}

#[test]
fn test_hsts_entry_is_not_expired_when_it_has_no_max_age() {
    let entry = HSTSEntry {
        host: "mozilla.org".to_string(),
        include_subdomains: false,
        max_age: None,
        timestamp: Some(time::get_time().sec as u64)
    };

    assert!(!entry.is_expired());
}

#[test]
fn test_hsts_entry_is_expired_when_it_has_reached_its_max_age() {
    let entry = HSTSEntry {
        host: "mozilla.org".to_string(),
        include_subdomains: false,
        max_age: Some(10),
        timestamp: Some(time::get_time().sec as u64 - 20u64)
    };

    assert!(entry.is_expired());
}

#[test]
fn test_hsts_entry_cant_be_created_with_ipv6_address_as_host() {
    let entry = HSTSEntry::new(
        "2001:0db8:0000:0000:0000:ff00:0042:8329".to_string(), IncludeSubdomains::NotIncluded, None
    );

    assert!(entry.is_none(), "able to create HSTSEntry with IPv6 host");
}

#[test]
fn test_hsts_entry_cant_be_created_with_ipv4_address_as_host() {
    let entry = HSTSEntry::new(
        "4.4.4.4".to_string(), IncludeSubdomains::NotIncluded, None
    );

    assert!(entry.is_none(), "able to create HSTSEntry with IPv4 host");
}

#[test]
fn test_push_entry_with_0_max_age_evicts_entry_from_list() {
    let mut list = HSTSList {
        entries: vec!(HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::NotIncluded, Some(500000u64)).unwrap())
    };

    list.push(HSTSEntry::new("mozilla.org".to_string(),
        IncludeSubdomains::NotIncluded, Some(0)).unwrap());

    assert!(list.is_host_secure("mozilla.org") == false)
}

#[test]
fn test_push_entry_to_hsts_list_should_not_add_subdomains_whose_superdomain_is_already_matched() {
    let mut list = HSTSList {
        entries: vec!(HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::Included, None).unwrap())
    };

    list.push(HSTSEntry::new("servo.mozilla.org".to_string(),
        IncludeSubdomains::NotIncluded, None).unwrap());

    assert!(list.entries.len() == 1)
}

#[test]
fn test_push_entry_to_hsts_list_should_update_existing_domain_entrys_include_subdomains() {
    let mut list = HSTSList {
        entries: vec!(HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::Included, None).unwrap())
    };

    assert!(list.is_host_secure("servo.mozilla.org"));

    list.push(HSTSEntry::new("mozilla.org".to_string(),
        IncludeSubdomains::NotIncluded, None).unwrap());

    assert!(!list.is_host_secure("servo.mozilla.org"))
}

#[test]
fn test_push_entry_to_hsts_list_should_not_create_duplicate_entry() {
    let mut list = HSTSList {
        entries: vec!(HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::NotIncluded, None).unwrap())
    };

    list.push(HSTSEntry::new("mozilla.org".to_string(),
        IncludeSubdomains::NotIncluded, None).unwrap());

    assert!(list.entries.len() == 1)
}

#[test]
fn test_push_multiple_entrie_to_hsts_list_should_add_them_all() {
    let mut list = HSTSList {
        entries: Vec::new()
    };

    assert!(!list.is_host_secure("mozilla.org"));
    assert!(!list.is_host_secure("bugzilla.org"));

    list.push(HSTSEntry::new("mozilla.org".to_string(),
        IncludeSubdomains::Included, None).unwrap());
    list.push(HSTSEntry::new("bugzilla.org".to_string(),
        IncludeSubdomains::Included, None).unwrap());

    assert!(list.is_host_secure("mozilla.org"));
    assert!(list.is_host_secure("bugzilla.org"));
}

#[test]
fn test_push_entry_to_hsts_list_should_add_an_entry() {
    let mut list = HSTSList {
        entries: Vec::new()
    };

    assert!(!list.is_host_secure("mozilla.org"));

    list.push(HSTSEntry::new("mozilla.org".to_string(),
        IncludeSubdomains::Included, None).unwrap());

    assert!(list.is_host_secure("mozilla.org"));
}

#[test]
fn test_parse_hsts_preload_should_return_none_when_json_invalid() {
    let mock_preload_content = "derp";
    assert!(HSTSList::new_from_preload(mock_preload_content).is_none(), "invalid preload list should not have parsed")
}

#[test]
fn test_parse_hsts_preload_should_return_none_when_json_contains_no_entries_key() {
    let mock_preload_content = "{\"nothing\": \"to see here\"}";
    assert!(HSTSList::new_from_preload(mock_preload_content).is_none(), "invalid preload list should not have parsed")
}

#[test]
fn test_parse_hsts_preload_should_decode_host_and_includes_subdomains() {
    let mock_preload_content = "{\
                                     \"entries\": [\
                                        {\"host\": \"mozilla.org\",\
                                         \"include_subdomains\": false}\
                                     ]\
                                 }";
    let hsts_list = HSTSList::new_from_preload(mock_preload_content);
    let entries = hsts_list.unwrap().entries;

    assert_eq!(entries[0].host, "mozilla.org");
    assert!(!entries[0].include_subdomains);
}

#[test]
fn test_hsts_list_with_no_entries_does_not_is_host_secure() {
    let hsts_list = HSTSList {
        entries: Vec::new()
    };

    assert!(!hsts_list.is_host_secure("mozilla.org"));
}

#[test]
fn test_hsts_list_with_exact_domain_entry_is_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::NotIncluded, None).unwrap()]
    };

    assert!(hsts_list.is_host_secure("mozilla.org"));
}

#[test]
fn test_hsts_list_with_subdomain_when_include_subdomains_is_true_is_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::Included, None).unwrap()]
    };

    assert!(hsts_list.is_host_secure("servo.mozilla.org"));
}

#[test]
fn test_hsts_list_with_subdomain_when_include_subdomains_is_false_is_not_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::NotIncluded, None).unwrap()]
    };

    assert!(!hsts_list.is_host_secure("servo.mozilla.org"));
}

#[test]
fn test_hsts_list_with_subdomain_when_host_is_not_a_subdomain_is_not_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::Included, None).unwrap()]
    };

    assert!(!hsts_list.is_host_secure("servo-mozilla.org"));
}

#[test]
fn test_hsts_list_with_subdomain_when_host_is_exact_match_is_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry::new("mozilla.org".to_string(),
            IncludeSubdomains::Included, None).unwrap()]
    };

    assert!(hsts_list.is_host_secure("mozilla.org"));
}

#[test]
fn test_hsts_list_with_expired_entry_is_not_is_host_secure() {
    let hsts_list = HSTSList {
        entries: vec![HSTSEntry {
            host: "mozilla.org".to_string(),
            include_subdomains: false,
            max_age: Some(20),
            timestamp: Some(time::get_time().sec as u64 - 100u64)
        }]
    };

    assert!(!hsts_list.is_host_secure("mozilla.org"));
}

#[test]
fn test_preload_hsts_domains_well_formed() {
    let hsts_list = preload_hsts_domains().unwrap();
    assert!(!hsts_list.entries.is_empty());
}

#[test]
fn test_secure_url_does_not_change_explicit_port() {
    let url = Url::parse("http://mozilla.org:8080/").unwrap();
    let secure = secure_url(&url);

    assert!(secure.port().unwrap() == 8080u16);
}

#[test]
fn test_secure_url_does_not_affect_non_http_schemas() {
    let url = Url::parse("file://mozilla.org").unwrap();
    let secure = secure_url(&url);

    assert_eq!(&secure.scheme, "file");
}

#[test]
fn test_secure_url_forces_an_http_host_in_list_to_https() {
    let url = Url::parse("http://mozilla.org").unwrap();
    let secure = secure_url(&url);

    assert_eq!(&secure.scheme, "https");
}
