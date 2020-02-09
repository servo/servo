/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

[Exposed=(Window,Worker)]
// https://drafts.fxtf.org/geometry/#domrect
interface DOMRect : DOMRectReadOnly {
    [Throws] constructor(optional unrestricted double x = 0, optional unrestricted double y = 0,
                optional unrestricted double width = 0, optional unrestricted double height = 0);
    inherit attribute unrestricted double x;
    inherit attribute unrestricted double y;
    inherit attribute unrestricted double width;
    inherit attribute unrestricted double height;
};
