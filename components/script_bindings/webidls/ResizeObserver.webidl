/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/resize-observer/#resize-observer-interface

[Pref="dom.resize_observer.enabled", Exposed=(Window)]
interface ResizeObserver {
    constructor(ResizeObserverCallback callback);
    undefined observe(Element target, optional ResizeObserverOptions options = {});
    undefined unobserve(Element target);
    undefined disconnect();
};

enum ResizeObserverBoxOptions {
    "border-box", "content-box", "device-pixel-content-box"
};

dictionary ResizeObserverOptions {
    ResizeObserverBoxOptions box = "content-box";
};

callback ResizeObserverCallback = undefined (sequence<ResizeObserverEntry> entries, ResizeObserver observer);
