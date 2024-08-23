/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://w3c.github.io/resource-timing/
 */

// https://w3c.github.io/resource-timing/#sec-performanceresourcetiming
[Exposed=(Window,Worker)]
interface PerformanceResourceTiming : PerformanceEntry {
    readonly attribute DOMString           initiatorType;
    readonly attribute DOMString           nextHopProtocol;
    // readonly attribute DOMHighResTimeStamp workerStart;
    readonly attribute DOMHighResTimeStamp redirectStart;
    readonly attribute DOMHighResTimeStamp redirectEnd;
    readonly attribute DOMHighResTimeStamp fetchStart;
    readonly attribute DOMHighResTimeStamp domainLookupStart;
    readonly attribute DOMHighResTimeStamp domainLookupEnd;
    readonly attribute DOMHighResTimeStamp connectStart;
    readonly attribute DOMHighResTimeStamp connectEnd;
    readonly attribute DOMHighResTimeStamp secureConnectionStart;
    readonly attribute DOMHighResTimeStamp requestStart;
    readonly attribute DOMHighResTimeStamp responseStart;
    readonly attribute DOMHighResTimeStamp responseEnd;
    readonly attribute unsigned long long  transferSize;
    readonly attribute unsigned long long  encodedBodySize;
    readonly attribute unsigned long long  decodedBodySize;
    [Default] object toJSON();
};
