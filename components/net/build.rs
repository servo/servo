/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

fn main() {
    let version =
        std::env::var("DEP_OPENSSL_VERSION_NUMBER").expect("missing DEP_OPENSSL_VERSION_NUMBER");
    let actual = u64::from_str_radix(&version, 16).unwrap();
    let minimum = 0x10101000;
    if actual < minimum {
        panic!(
            "Your OpenSSL version is older than 1.1.1 ({:x}), you have: {:x}",
            minimum, actual
        );
    }
}
