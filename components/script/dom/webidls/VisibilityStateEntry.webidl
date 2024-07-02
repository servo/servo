/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#visibilitystateentry

[Exposed=(Window)]
interface VisibilityStateEntry : PerformanceEntry {
  readonly attribute DOMString name;                 // shadows inherited name
  readonly attribute DOMString entryType;            // shadows inherited entryType
  readonly attribute DOMHighResTimeStamp startTime;  // shadows inherited startTime
  readonly attribute unsigned long duration;         // shadows inherited duration
};
