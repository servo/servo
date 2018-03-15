/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#sec-window.performance-attribute
 */

typedef double DOMHighResTimeStamp;
typedef sequence<PerformanceEntry> PerformanceEntryList;

[Exposed=(Window, Worker)]
interface Performance {
  DOMHighResTimeStamp now();
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
  void mark(DOMString markName);
  void clearMarks(optional DOMString markName);
  [Throws]
  void measure(DOMString measureName, optional DOMString startMark, optional DOMString endMark);
  void clearMeasures(optional DOMString measureName);
};
