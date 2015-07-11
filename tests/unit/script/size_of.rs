/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::dom::characterdata::CharacterData;
use script::dom::element::Element;
use script::dom::eventtarget::EventTarget;
use script::dom::htmldivelement::HTMLDivElement;
use script::dom::htmlelement::HTMLElement;
use script::dom::htmlspanelement::HTMLSpanElement;
use script::dom::node::Node;
use script::dom::text::Text;

use std::mem::size_of;

// Macro so that we can stringify type names
// I'd really prefer the tests themselves to be run at plugin time,
// however rustc::middle doesn't have access to the full type data
macro_rules! sizeof_checker (
    ($testname: ident, $t:ty, $known_size:expr) => (
        #[test]
        fn $testname() {
            let new = size_of::<$t>();
            let old = $known_size;
            if new < old {
                panic!("Your changes have decreased the stack size of commonly used DOM struct {} from {} to {}. \
                        Good work! Please update the size in script/tests.rs",
                        stringify!($t), old, new)
            } else if new > old {
                panic!("Your changes have increased the stack size of commonly used DOM struct {} from {} to {}. \
                        These structs are present in large quantities in the DOM, and increasing the size \
                        may dramatically affect our memory footprint. Please consider choosing a design which \
                        avoids this increase. If you feel that the increase is necessary, \
                        update to the new size in script/tests.rs.",
                        stringify!($t), old, new)
        }
    });
);

// Update the sizes here
sizeof_checker!(size_event_target, EventTarget, 40);
sizeof_checker!(size_node, Node, 168);
sizeof_checker!(size_element, Element, 280);
sizeof_checker!(size_htmlelement, HTMLElement, 296);
sizeof_checker!(size_div, HTMLDivElement, 296);
sizeof_checker!(size_span, HTMLSpanElement, 296);
sizeof_checker!(size_text, Text, 200);
sizeof_checker!(size_characterdata, CharacterData, 200);
