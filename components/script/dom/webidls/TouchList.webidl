/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://w3c.github.io/touch-events/#idl-def-TouchList
[Exposed=(Window,Worker)]
interface TouchList {
    readonly    attribute unsigned long length;
    getter Touch? item (unsigned long index);
};
