/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmlmeterelement
[HTMLConstructor]
interface HTMLMeterElement : HTMLElement {
  //         attribute double value;
  //         attribute double min;
  //         attribute double max;
  //         attribute double low;
  //         attribute double high;
  //         attribute double optimum;
  readonly attribute NodeList labels;
};
