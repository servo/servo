// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.7.2-3-s
description: >
    Strict Mode - TypeError is thrown if LeftHandSide is a reference
    to a non-writable data property
flags: [onlyStrict]
---*/

        var _8_7_2_3 = {};
        Object.defineProperty(_8_7_2_3, "b", {
            writable: false
        });
assert.throws(TypeError, function() {
            _8_7_2_3.b = 11;
});
