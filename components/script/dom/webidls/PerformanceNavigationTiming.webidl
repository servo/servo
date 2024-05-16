/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming
 */

enum NavigationTimingType {
    "navigate",
    "reload",
    "back_forward",
    "prerender"
};

[Exposed=Window]
interface PerformanceNavigationTiming : PerformanceResourceTiming {
    readonly attribute DOMHighResTimeStamp  unloadEventStart;
    readonly attribute DOMHighResTimeStamp  unloadEventEnd;
    readonly attribute DOMHighResTimeStamp  domInteractive;
    readonly attribute DOMHighResTimeStamp  domContentLoadedEventStart;
    readonly attribute DOMHighResTimeStamp  domContentLoadedEventEnd;
    readonly attribute DOMHighResTimeStamp  domComplete;
    readonly attribute DOMHighResTimeStamp  loadEventStart;
    readonly attribute DOMHighResTimeStamp  loadEventEnd;
    readonly attribute NavigationTimingType type;
    readonly attribute unsigned short       redirectCount;
    [Default] object toJSON();
    /* Servo-only attribute for measuring when the top-level document (not iframes) is complete. */
    [Pref="dom.testperf.enabled"]
    readonly attribute DOMHighResTimeStamp  topLevelDomComplete;
};
