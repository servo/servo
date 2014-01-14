/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Number converter,
// This file will be move from main/layout to style/

use std::hashmap::HashMap;

use style::computed_values::list_style_type;

pub trait Roman {
    fn to_roman(&self, number: int) -> ~str;
}

impl Roman for HashMap<int, ~str> {
    fn to_roman(&self, number: int) -> ~str {
        let base = ~[1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        let mut num = number;
        let mut result = ~"";
        for &i in base.iter() {
            while num >= i {
                match self.find(&i) {
                    None => fail!("key is not matched."),
                    Some(v) => {
                        num -= i;
                        result.push_str(*v);
                    }
                }
            }
        }
        return result;
    }
}

struct RomanNumber {
    upper_romans: HashMap<int, ~str>,
    lower_romans: HashMap<int, ~str>
}

impl RomanNumber {
    fn new() -> RomanNumber {
        RomanNumber {
            upper_romans: init_upper_roman(),
            lower_romans: init_lower_roman()
        }
    }

    fn to_upper(&self, number: int) -> ~str {
        self.upper_romans.to_roman(number)
    }

    fn to_lower(&self, number: int) -> ~str {
        self.lower_romans.to_roman(number)
    }
}

fn init_upper_roman() -> HashMap<int, ~str> {
    let mut map: HashMap<int, ~str> = HashMap::new();//with_capacity(20);
    map.insert(1, ~"\u2160");
    map.insert(2, ~"\u2161");
    map.insert(3, ~"\u2162");
    map.insert(4, ~"\u2163");
    map.insert(5, ~"\u2164");
    map.insert(6, ~"\u2165");
    map.insert(7, ~"\u2166");
    map.insert(8, ~"\u2167");
    map.insert(9, ~"\u2168");
    map.insert(10, ~"\u2169");
    //map.insert(11, ~"\u216A");     // http://www.fileformat.info/info/unicode/char/216A/index.htm
    //map.insert(12, ~"\u216B");     // http://www.fileformat.info/info/unicode/char/216B/index.htm
    map.insert(40, ~"\u2169\u216C");
    map.insert(50, ~"\u216C");       // http://www.fileformat.info/info/unicode/char/216C/index.htm
    map.insert(90, ~"\u2169\u216D");
    map.insert(100, ~"\u216D");      // Unicode Character 'ROMAN NUMERAL ONE HUNDRED' (U+216D)
    map.insert(400, ~"\u216C\u216E");
    map.insert(500, ~"\u216E");      // Unicode Character 'ROMAN NUMERAL FIVE HUNDRED' (U+216E)
    map.insert(900, ~"\u216D\u216F");
    map.insert(1000, ~"\u216F");     //Unicode Character 'ROMAN NUMERAL ONE THOUSAND' (U+216F)

    return map;
}

fn init_lower_roman() -> HashMap<int, ~str> {
    let mut map: HashMap<int, ~str> = HashMap::new();//with_capacity(20);
    map.insert(1, ~"\u2170");        // Unicode Character 'SMALL ROMAN NUMERAL ONE' (U+2170)
    map.insert(2, ~"\u2171");
    map.insert(3, ~"\u2172");
    map.insert(4, ~"\u2173");
    map.insert(5, ~"\u2174");
    map.insert(6, ~"\u2175");
    map.insert(7, ~"\u2176");
    map.insert(8, ~"\u2177");
    map.insert(9, ~"\u2178");
    map.insert(10, ~"\u2179");
    //map.insert(11, ~"\u217A");      // Unicode Character 'SMALL ROMAN NUMERAL ELEVEN' (U+217A)
    //map.insert(12, ~"\u217B");      // Unicode Character 'SMALL ROMAN NUMERAL TWELVE' (U+217B)
    map.insert(40, ~"\u2179\u217C");
    map.insert(50, ~"\u217C");       // http://www.fileformat.info/info/unicode/char/217C/index.htm
    map.insert(90, ~"\u2179\u217D");
    map.insert(100, ~"\u217D");      // Unicode Character 'SMALL ROMAN NUMERAL ONE HUNDRED' (U+217D)
    map.insert(400, ~"\u217C\u217E");
    map.insert(500, ~"\u217E");      // Unicode Character 'SMALL ROMAN NUMERAL FIVE HUNDRED' (U+217E)
    map.insert(900, ~"\u217D\u217F");
    map.insert(1000, ~"\u217F");     //Unicode Character 'SMALL ROMAN NUMERAL ONE THOUSAND' (U+217F)

    return map;
}

pub struct Numbers {
    roman: RomanNumber,
}

impl Numbers {
    pub fn new() -> Numbers {
        Numbers { roman: RomanNumber::new() }
    }

    /// generate list_style_type.
    pub fn to_list_style_type(&self, style_type: list_style_type::T, sequence: int) -> ~str {

        // if sequence is minus or zero
        if sequence <= 0 {
            return sequence.to_str() + self.dot_space();
        }

        match style_type {

            //Ordered
            //list_style_type::decimal_leading_zero => self.to_decimal_leading_zero(sequence) + self.dot_space(), 
            list_style_type::decimal     => { sequence.to_str() + self.dot_space() },
            list_style_type::lower_roman => self.to_lower_roman(sequence) + self.dot_space(),
            list_style_type::upper_roman => self.to_upper_roman(sequence) + self.dot_space(),
            //list_style_type::lower_greek => self.to_lower_greek(sequence) + self.dot_space(),
            //list_style_type::lower_latin => self.to_lower_latin(sequence) + self.dot_space(),
            //list_style_type::upper_latin => self.to_upper_latin(sequence) + self.dot_space(),
            //list_style_type::lower_alpha => self.to_lower_alpha(sequence) + self.dot_space(),
            //list_style_type::upper_alpha => self.to_upper_alpha(sequence) + self.dot_space(),
            //list_style_type::armenian    => self.to_armenian(sequence) + self.dot_space(),
            //list_style_type::georgian    => self.to_georgian(sequence) + self.dot_space(),
            
            //UnOrdered
            list_style_type::circle      => ~"\u25CB" + self.space(), // ○
            list_style_type::disc        => ~"\u25CF" + self.space(), // ●
            list_style_type::square      => ~"\u25A0" + self.space(), // ■

            list_style_type::none        => ~"", // type is none
        }
    }
    pub fn dot_space(&self) -> ~str {
        ~"\u002E" + self.space()
    }
    pub fn space(&self) -> ~str {
        ~"\u2009"
    }
    pub fn to_decimal_leading_zero(&self, _number: int) -> ~str {
        fail!("TODO: decimal_leading_zero");
    }
    pub fn to_upper_roman(&self, number: int) -> ~str {
        self.roman.to_upper(number)     
    }
    pub fn to_lower_roman(&self, number: int) -> ~str {
        self.roman.to_lower(number)     
    }
    pub fn to_upper_alpha(&self, _number: int) -> ~str {
        fail!("TODO: upper-alpha");
    }
    pub fn to_lower_alpha(&self, _number: int) -> ~str {
        fail!("TODO: lower-alpha");
    }
    pub fn to_upper_latin(&self, _number: int) -> ~str {
        fail!("TODO: upper-latin");
    }
    pub fn to_lower_latin(&self, _number: int) -> ~str {
        fail!("TODO: lower-latin");
    }
    pub fn to_upper_greek(&self, _number: int) -> ~str {
        fail!("there is no upper-greek property");
    }
    pub fn to_lower_greek(&self, _number: int) -> ~str {
        fail!("TODO: lower-greek");
    }
    pub fn to_armenian(&self, _number: int) -> ~str {
        fail!("TODO: armenian");
    }
    pub fn to_georgian(&self, _number: int) -> ~str {
        fail!("TODO: georgian");
    }
}
