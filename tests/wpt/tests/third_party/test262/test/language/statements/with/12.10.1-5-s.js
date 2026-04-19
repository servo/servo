// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10.1-5-s
description: >
    with statement allowed in nested Function even if its container
    Function is strict)
flags: [onlyStrict]
---*/

    Function("\'use strict\'; var f1 = Function( \"var o = {}; with (o) {};\")");
