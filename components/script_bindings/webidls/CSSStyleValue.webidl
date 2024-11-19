/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-typed-om-1/#cssstylevalue
// NOTE: should this be exposed to Window?
[Pref="dom.worklet.enabled", Exposed=(Worklet)]
interface CSSStyleValue {
    stringifier;
};
