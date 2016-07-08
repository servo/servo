/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * http://dev.w3.org/fxtf/geometry/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// http://dev.w3.org/fxtf/geometry/Overview.html#dompoint
[Constructor(optional unrestricted double x = 0, optional unrestricted double y = 0,
             optional unrestricted double z = 0, optional unrestricted double w = 1),
 Exposed=(Window,Worker)]
interface DOMPoint : DOMPointReadOnly {
    inherit attribute unrestricted double x;
    inherit attribute unrestricted double y;
    inherit attribute unrestricted double z;
    inherit attribute unrestricted double w;
};

dictionary DOMPointInit {
    unrestricted double x = 0;
    unrestricted double y = 0;
    unrestricted double z = 0;
    unrestricted double w = 1;
};
