/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use url::Url;

#[test]
fn load_htm() {
    let mut path = env::current_dir().expect("didn't get working dir");
    path.push("tests/test.jpeg");

    let canon_path = path.canonicalize().expect("file path doesn't exist");
    let url = Url::from_file_path(canon_path);

    assert!(url.is_ok());
}
