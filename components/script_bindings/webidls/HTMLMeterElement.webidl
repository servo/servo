/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlmeterelement
[Exposed=Window]
interface HTMLMeterElement : HTMLElement {
  [HTMLConstructor] constructor();

  [CEReactions]
           attribute double value;
  [CEReactions]
           attribute double min;
  [CEReactions]
           attribute double max;
  [CEReactions]
           attribute double low;
  [CEReactions]
           attribute double high;
  [CEReactions]
           attribute double optimum;
  readonly attribute NodeList labels;
};
