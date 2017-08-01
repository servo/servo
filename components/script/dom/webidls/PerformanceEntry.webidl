/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/performance-timeline/#the-performanceentry-interface
 */

[Exposed=(Window,Worker)]
interface PerformanceEntry {
  readonly attribute DOMString           name;
  readonly attribute DOMString           entryType;
  readonly attribute DOMHighResTimeStamp startTime;
  readonly attribute DOMHighResTimeStamp duration;

  // [Default] object toJSON();
};
