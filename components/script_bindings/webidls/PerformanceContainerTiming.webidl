/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://wicg.github.io/container-timing/#sec-performance-container-timing
 */

[Exposed=Window, Pref="container_timing_enabled"]
interface PerformanceContainerTiming : PerformanceEntry {
    readonly attribute DOMString identifier;
    readonly attribute DOMRectReadOnly intersectionRect;
    readonly attribute unsigned long long size;
    readonly attribute DOMHighResTimeStamp firstRenderTime;
    // TODO: PaintTimingMixin's `presentationTime` is not implemented; `paintTime`
    // always equals `startTime` for now.
    readonly attribute DOMHighResTimeStamp paintTime;
    readonly attribute Element? lastPaintedElement;
    readonly attribute Element? rootElement;
    [Default] object toJSON();
};
