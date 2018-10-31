/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage#time-ranges

[Exposed=Window]
interface TimeRanges {
  readonly attribute unsigned long length;
  [Throws] double start(unsigned long index);
  [Throws] double end(unsigned long index);
};
