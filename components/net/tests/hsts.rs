/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::time::Duration as StdDuration;

use base64::Engine;
use net::hsts::{HstsEntry, HstsList, HstsPreloadList};
use net_traits::IncludeSubdomains;

#[test]
fn test_hsts_entry_is_not_expired_when_it_has_no_expires_at() {
    let entry = HstsEntry {
        host: "example.com".to_owned(),
        include_subdomains: false,
        expires_at: None,
    };

    assert!(!entry.is_expired());
}

#[test]
fn test_hsts_entry_is_expired_when_it_has_reached_its_max_age() {
    let entry = HstsEntry {
        host: "example.com".to_owned(),
        include_subdomains: false,
        expires_at: Some(NonZeroU64::new(1).unwrap()),
    };

    assert!(entry.is_expired());
}

#[test]
fn test_hsts_entry_cant_be_created_with_ipv6_address_as_host() {
    let entry = HstsEntry::new(
        "2001:0db8:0000:0000:0000:ff00:0042:8329".to_owned(),
        IncludeSubdomains::NotIncluded,
        None,
    );

    assert!(entry.is_none(), "able to create HstsEntry with IPv6 host");
}

#[test]
fn test_hsts_entry_cant_be_created_with_ipv4_address_as_host() {
    let entry = HstsEntry::new("4.4.4.4".to_owned(), IncludeSubdomains::NotIncluded, None);

    assert!(entry.is_none(), "able to create HstsEntry with IPv4 host");
}

#[test]
fn test_base_domain_in_entries_map() {
    let entries_map = HashMap::new();

    let mut list = HstsList {
        entries_map: entries_map,
    };

    list.push(
        HstsEntry::new(
            "servo.example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );
    list.push(
        HstsEntry::new(
            "firefox.example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );
    list.push(
        HstsEntry::new(
            "example.org".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );

    assert_eq!(list.entries_map.len(), 2);
    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 2);
}

#[test]
fn test_push_entry_with_0_max_age_is_not_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                Some(StdDuration::from_secs(500000)),
            )
            .unwrap(),
        ],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    list.push(
        HstsEntry::new(
            "example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            Some(StdDuration::ZERO),
        )
        .unwrap(),
    );

    assert_eq!(list.is_host_secure("example.com"), false)
}

fn test_push_entry_with_0_max_age_evicts_entry_from_list() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                Some(StdDuration::from_secs(500000)),
            )
            .unwrap(),
        ],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 1);

    list.push(
        HstsEntry::new(
            "example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            Some(StdDuration::ZERO),
        )
        .unwrap(),
    );
    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 0);
}

#[test]
fn test_push_entry_to_hsts_list_should_not_add_subdomains_whose_superdomain_is_already_matched() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap()],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    list.push(
        HstsEntry::new(
            "servo.example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );

    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 1)
}

#[test]
fn test_push_entry_to_hsts_list_should_add_subdomains_whose_superdomain_doesnt_include() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                None,
            )
            .unwrap(),
        ],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    list.push(
        HstsEntry::new(
            "servo.example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );

    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 2)
}

#[test]
fn test_push_entry_to_hsts_list_should_update_existing_domain_entrys_include_subdomains() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap()],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    assert!(list.is_host_secure("servo.example.com"));

    list.push(
        HstsEntry::new(
            "example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );

    assert!(!list.is_host_secure("servo.example.com"))
}

#[test]
fn test_push_entry_to_hsts_list_should_not_create_duplicate_entry() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                None,
            )
            .unwrap(),
        ],
    );
    let mut list = HstsList {
        entries_map: entries_map,
    };

    list.push(
        HstsEntry::new(
            "example.com".to_owned(),
            IncludeSubdomains::NotIncluded,
            None,
        )
        .unwrap(),
    );

    assert_eq!(list.entries_map.get("example.com").unwrap().len(), 1)
}

#[test]
fn test_push_multiple_entrie_to_hsts_list_should_add_them_all() {
    let mut list = HstsList {
        entries_map: HashMap::new(),
    };

    assert!(!list.is_host_secure("example.com"));
    assert!(!list.is_host_secure("example.org"));

    list.push(HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap());
    list.push(HstsEntry::new("example.org".to_owned(), IncludeSubdomains::Included, None).unwrap());

    assert!(list.is_host_secure("example.com"));
    assert!(list.is_host_secure("example.org"));
}

#[test]
fn test_push_entry_to_hsts_list_should_add_an_entry() {
    let mut list = HstsList {
        entries_map: HashMap::new(),
    };

    assert!(!list.is_host_secure("example.com"));

    list.push(HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap());

    assert!(list.is_host_secure("example.com"));
}

#[test]
fn test_parse_hsts_preload_should_return_none_when_json_invalid() {
    let mock_preload_content = "derp".as_bytes().to_vec();
    assert!(
        HstsPreloadList::from_preload(mock_preload_content).is_none(),
        "invalid preload list should not have parsed"
    )
}

#[test]
fn test_parse_hsts_preload_should_decode_host_and_includes_subdomains() {
    // Generated with `fst map --sorted` on a csv of "example.com,0\nexample.org,3"
    let mock_preload_content = base64::engine::general_purpose::STANDARD
        .decode("AwAAAAAAAAAAAAAAAAAAAAAQkMQAEJfHAwABBW9jEQLNws/J0MXqwgIAAAAAAAAAJwAAAAAAAADVOFe6")
        .unwrap();
    let hsts_list = HstsPreloadList::from_preload(mock_preload_content).unwrap();

    assert_eq!(hsts_list.is_host_secure("derp"), false);
    assert_eq!(hsts_list.is_host_secure("example.com"), true);
    assert_eq!(hsts_list.is_host_secure("servo.example.com"), false);
    assert_eq!(hsts_list.is_host_secure("example.org"), true);
    assert_eq!(hsts_list.is_host_secure("servo.example.org"), true);
}

#[test]
fn test_hsts_list_with_no_entries_map_does_not_is_host_secure() {
    let hsts_list = HstsList {
        entries_map: HashMap::new(),
    };

    assert!(!hsts_list.is_host_secure("example.com"));
}

#[test]
fn test_hsts_list_with_exact_domain_entry_is_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                None,
            )
            .unwrap(),
        ],
    );

    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(hsts_list.is_host_secure("example.com"));
}

#[test]
fn test_hsts_list_with_subdomain_when_include_subdomains_is_true_is_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap()],
    );
    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(hsts_list.is_host_secure("servo.example.com"));
}

#[test]
fn test_hsts_list_with_subdomain_when_include_subdomains_is_false_is_not_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![
            HstsEntry::new(
                "example.com".to_owned(),
                IncludeSubdomains::NotIncluded,
                None,
            )
            .unwrap(),
        ],
    );
    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(!hsts_list.is_host_secure("servo.example.com"));
}

#[test]
fn test_hsts_list_with_subdomain_when_host_is_not_a_subdomain_is_not_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap()],
    );
    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(!hsts_list.is_host_secure("servo-example.com"));
}

#[test]
fn test_hsts_list_with_subdomain_when_host_is_exact_match_is_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry::new("example.com".to_owned(), IncludeSubdomains::Included, None).unwrap()],
    );
    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(hsts_list.is_host_secure("example.com"));
}

#[test]
fn test_hsts_list_with_expired_entry_is_not_is_host_secure() {
    let mut entries_map = HashMap::new();
    entries_map.insert(
        "example.com".to_owned(),
        vec![HstsEntry {
            host: "example.com".to_owned(),
            include_subdomains: false,
            expires_at: Some(NonZeroU64::new(1).unwrap()),
        }],
    );
    let hsts_list = HstsList {
        entries_map: entries_map,
    };

    assert!(!hsts_list.is_host_secure("example.com"));
}

#[test]
fn test_preload_hsts_domains_well_formed() {
    let hsts_list = HstsPreloadList::from_servo_preload();
    assert_ne!(hsts_list.0.len(), 0);
}
