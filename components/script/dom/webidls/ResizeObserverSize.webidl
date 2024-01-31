/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/resize-observer/#resizeobserversize

[Pref="dom.resize_observer.enabled", Exposed=Window]
interface ResizeObserverSize {
    readonly attribute unrestricted double inlineSize;
    readonly attribute unrestricted double blockSize;
};
