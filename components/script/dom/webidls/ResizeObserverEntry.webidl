/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 // https://drafts.csswg.org/resize-observer/#resize-observer-entry-interface

[Pref="dom.resize_observer.enabled", Exposed=Window]
interface ResizeObserverEntry {
    readonly attribute Element target;
    readonly attribute DOMRectReadOnly contentRect;
    readonly attribute /*FrozenArray<ResizeObserverSize>*/any borderBoxSize;
    readonly attribute /*FrozenArray<ResizeObserverSize>*/any contentBoxSize;
    readonly attribute /*FrozenArray<ResizeObserverSize>*/any devicePixelContentBoxSize;
};
