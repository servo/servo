/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-typed-om-1/#cssstylevalue
// NOTE: should this be exposed to Window?
[Exposed=(Worklet)]
interface CSSStyleValue {
    stringifier;
    // static CSSStyleValue? parse(DOMString property, DOMString cssText);
    // static sequence<CSSStyleValue>? parseAll(DOMString property, DOMString cssText);
    // This is a deprecated property, it's not in the spec any more but is used in houdini-samples
    readonly attribute DOMString cssText;
};
