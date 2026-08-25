/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/performance-timeline/#the-performanceobserver-interface
 */

dictionary PerformanceObserverInit {
  sequence<DOMString> entryTypes;
  DOMString type;
  boolean buffered;
};

callback PerformanceObserverCallback = undefined (PerformanceObserverEntryList entries, PerformanceObserver observer);

[Exposed=(Window,Worker)]
interface PerformanceObserver {
  [Throws] constructor(PerformanceObserverCallback callback);
  [Throws]
  undefined observe(optional PerformanceObserverInit options = {});
  undefined disconnect();
  PerformanceEntryList takeRecords();
  // codegen doesn't like SameObject+static and doesn't know FrozenArray
  /*[SameObject]*/ static readonly attribute /*FrozenArray<DOMString>*/ any supportedEntryTypes;
};
