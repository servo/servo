/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltitleelement
[Exposed=Window]
interface HTMLTitleElement : HTMLElement {
    [HTMLConstructor] constructor();

    [CEReactions, Pure]
    attribute DOMString text;
};
