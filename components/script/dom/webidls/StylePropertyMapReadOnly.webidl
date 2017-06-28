/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-typed-om-1/#stylepropertymapreadonly
[Exposed=(Window, Worklet)]
interface StylePropertyMapReadOnly {
    CSSStyleValue? get(DOMString property);
    // sequence<CSSStyleValue> getAll(DOMString property);
    boolean has(DOMString property);
    // iterable<DOMString, (CSSStyleValue or sequence<CSSStyleValue>)>;
    // sequence<DOMString> getProperties();
    // stringifier;
};
