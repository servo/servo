/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://drafts.fxtf.org/geometry/#DOMQuad
 *
 * Copyright:
 * To the extent possible under law, the editors have waived all copyright and
 * related or neighboring rights to this work.
 */

[Constructor(optional DOMPointInit p1, optional DOMPointInit p2,
             optional DOMPointInit p3, optional DOMPointInit p4),
 Exposed=(Window,Worker)]
interface DOMQuad {
    [NewObject] static DOMQuad fromRect(optional DOMRectInit other);
    [NewObject] static DOMQuad fromQuad(optional DOMQuadInit other);

    [SameObject] readonly attribute DOMPoint p1;
    [SameObject] readonly attribute DOMPoint p2;
    [SameObject] readonly attribute DOMPoint p3;
    [SameObject] readonly attribute DOMPoint p4;
    [NewObject] DOMRect getBounds();
};

dictionary DOMQuadInit {
    DOMPointInit p1;
    DOMPointInit p2;
    DOMPointInit p3;
    DOMPointInit p4;
};
