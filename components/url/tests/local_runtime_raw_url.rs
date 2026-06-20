/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Mutex;

use servo_url::{ServoUrl, local_runtime_raw_url_obfuscation_reason};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_package_mode() {
    unsafe {
        std::env::remove_var("SERVORENA_PACKAGE_ID");
        std::env::remove_var("SERVORENA_PACKAGE_ROOT");
    }
}

#[test]
fn raw_url_obfuscation_allows_ordinary_percent_encoding() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_package_mode();
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./assets/logo%20one.png"),
        None
    );
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./data/%7Bitem%7D.json"),
        None
    );
}

#[test]
fn raw_url_obfuscation_flags_path_boundary_encodings() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_package_mode();
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./%2e%2e/secret.txt"),
        Some("RawPercentEncodedDotDenied"),
    );
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./safe%2fsecret.txt"),
        Some("RawPercentEncodedSlashDenied"),
    );
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./safe%5csecret.txt"),
        Some("RawPercentEncodedBackslashDenied"),
    );
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./safe%00secret.txt"),
        Some("RawPercentEncodedNulDenied"),
    );
    assert_eq!(
        local_runtime_raw_url_obfuscation_reason("./%252e%252e/secret.txt"),
        Some("RawDoubleEncodedPathObfuscationDenied"),
    );
}

#[test]
fn url_join_normalizes_percent_encoded_dot_dot_before_late_path_gate() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_package_mode();
    let base = ServoUrl::parse("asset://com.example.app/app/index.html").unwrap();
    let joined = base.join("./%2e%2e/secret.txt").unwrap();

    assert_eq!(joined.as_str(), "asset://com.example.app/secret.txt");
    assert_eq!(joined.path(), "/secret.txt");
}

#[test]
fn package_mode_denies_raw_obfuscated_relative_url_before_normalization() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe {
        std::env::set_var("SERVORENA_PACKAGE_ID", "com.example.app");
        std::env::set_var("SERVORENA_PACKAGE_ROOT", "/tmp/servo-local-runtime-test");
    }
    let base = ServoUrl::parse("asset://com.example.app/app/index.html").unwrap();

    assert!(base.join("./%2e%2e/secret.txt").is_err());
    assert!(ServoUrl::parse_with_base(Some(&base), "./%252e%252e/secret.txt").is_err());

    unsafe {
        std::env::remove_var("SERVORENA_PACKAGE_ID");
        std::env::remove_var("SERVORENA_PACKAGE_ROOT");
    }
}
