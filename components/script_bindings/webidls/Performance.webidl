/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/hr-time/#sec-performance
 */

typedef double DOMHighResTimeStamp;
typedef sequence<PerformanceEntry> PerformanceEntryList;

[Exposed=(Window, Worker)]
interface Performance : EventTarget {
  DOMHighResTimeStamp now();
  readonly attribute DOMHighResTimeStamp timeOrigin;
  [Default] object toJSON();
};

// https://w3c.github.io/performance-timeline/#extensions-to-the-performance-interface
[Exposed=(Window, Worker)]
partial interface Performance {
  PerformanceEntryList getEntries();
  PerformanceEntryList getEntriesByType(DOMString type);
  PerformanceEntryList getEntriesByName(DOMString name,
                                        optional DOMString type);
};

// https://w3c.github.io/user-timing/#extensions-performance-interface
[Exposed=(Window,Worker)]
partial interface Performance {
  [Throws]
  undefined mark(DOMString markName);
  undefined clearMarks(optional DOMString markName);
  [Throws]
  undefined measure(DOMString measureName, optional DOMString startMark, optional DOMString endMark);
  undefined clearMeasures(optional DOMString measureName);
};

//https://w3c.github.io/resource-timing/#sec-extensions-performance-interface
partial interface Performance {
  undefined clearResourceTimings ();
  undefined setResourceTimingBufferSize (unsigned long maxSize);
              attribute EventHandler onresourcetimingbufferfull;
};

// https://w3c.github.io/navigation-timing/#extensions-to-the-performance-interface
[Exposed=Window]
partial interface Performance {
  [SameObject]
  readonly attribute PerformanceNavigationTiming timing;
  [SameObject]
  readonly attribute PerformanceNavigation navigation;
};
