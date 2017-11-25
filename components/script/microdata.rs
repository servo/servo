/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 
//extern crate serde_json;
use std::collections::HashMap;
pub struct Microdata {}

impl Microdata {
    //[Pref="dom.microdata.testing.enabled"]
    pub fn parse() -> bool {
        println!("Hello");
        let mut book_reviews = HashMap::new();
        let mut rating = HashMap::new();

        rating.insert("a", "1");
        rating.insert("b", "2");
        rating.insert("c", "3");
        rating.insert("d", "4");

        book_reviews.insert("Adventures of Huckleberry Finn", rating);

        //let j = serde_json::to_string(&book_reviews);

        // Print, write to a file, or send to an HTTP server.
        println!("from microdata module");
        true
    }
}
