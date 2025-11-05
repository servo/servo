/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/largest-contentful-paint/#sec-largest-contentful-paint-interface
 */

[Exposed=Window]
interface LargestContentfulPaint : PerformanceEntry {
    readonly attribute DOMHighResTimeStamp loadTime;
    readonly attribute DOMHighResTimeStamp renderTime;
    readonly attribute unsigned long size;
    readonly attribute Element? element;
};
