/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-paint-api/#paintsize
[Pref="dom.worklet.enabled", Exposed=PaintWorklet]
interface PaintSize {
    readonly attribute double width;
    readonly attribute double height;
};
