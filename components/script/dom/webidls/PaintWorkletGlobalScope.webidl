/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://drafts.css-houdini.org/css-paint-api/#paintworkletglobalscope
[Global=(Worklet,PaintWorklet), Pref="dom.worklet.enabled", Exposed=PaintWorklet]
interface PaintWorkletGlobalScope : WorkletGlobalScope {
    [Throws] void registerPaint(DOMString name, VoidFunction paintCtor);
    // This function is to be used only for testing, and should not be
    // accessible outside of that use.
    [Pref="dom.worklet.blockingsleep.enabled"]
    void sleep(unsigned long long ms);
};
