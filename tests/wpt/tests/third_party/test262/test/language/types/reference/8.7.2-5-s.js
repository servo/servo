// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.7.2-5-s
description: >
    Strict Mode - TypeError is thrown if LeftHandSide is a reference
    to a non-existent property of an non-extensible object
flags: [onlyStrict]
---*/

        var _8_7_2_5 = {};
        Object.preventExtensions(_8_7_2_5);
assert.throws(TypeError, function() {
            _8_7_2_5.b = 11;
});
