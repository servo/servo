/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.fxtf.org/geometry-1/#domrectlist

[Exposed=Window]
interface DOMRectList {
    readonly attribute unsigned long length;
    getter DOMRect? item(unsigned long index);
};
