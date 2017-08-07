/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-typed-om-1/#stylepropertymapreadonly
// NOTE: should this be exposed to Window?
[Pref="dom.worklet.enabled", Exposed=(Worklet)]
interface StylePropertyMapReadOnly {
    CSSStyleValue? get(DOMString property);
    // sequence<CSSStyleValue> getAll(DOMString property);
    boolean has(DOMString property);
    // iterable<DOMString, (CSSStyleValue or sequence<CSSStyleValue>)>;
    sequence<DOMString> getProperties();
    // https://github.com/w3c/css-houdini-drafts/issues/268
    // stringifier;
};
