/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


// This puts a "multi-line string
// inside of a comment" and then subsequently has a hyphenated-phrase


const FOO: &'static str = "Do not confuse 'apostrophes',
    They can be 'lifetimes' or 'characters'";


fn main() {
    assert!(foo("test
                 foo-bar"));

    assert!(foo("test
                 test2 \"
                 foo-bar"));

    assert!(foo("test
                 test2 \
                 foo-bar"));

    println!("This is a multiline string with a URL, which kinda, \
             sorta looks like a comment https://github.com/servo/servo/");
}
