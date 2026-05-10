// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.1-4-14-s
description: >
    simple assignment throws TypeError if LeftHandSide is a readonly
    property in strict mode (Number.MAX_VALUE)
flags: [onlyStrict]
---*/


assert.throws(TypeError, function() {
    Number.MAX_VALUE = 42;
});
