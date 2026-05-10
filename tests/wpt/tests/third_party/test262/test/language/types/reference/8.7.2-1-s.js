// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.7.2-1-s
description: >
    Strict Mode - ReferenceError is thrown if LeftHandSide evaluates
    to an unresolvable Reference
flags: [onlyStrict]
---*/


assert.throws(ReferenceError, function() {
            eval("_8_7_2_1 = 11;");
});
