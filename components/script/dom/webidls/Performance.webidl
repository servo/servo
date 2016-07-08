/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#sec-window.performance-attribute
 */

typedef double DOMHighResTimeStamp;

[Exposed=(Window,Worker)]
interface Performance {
  readonly attribute PerformanceTiming timing;
  /*  readonly attribute PerformanceNavigation navigation; */
};

partial interface Performance {
  DOMHighResTimeStamp now();
};
