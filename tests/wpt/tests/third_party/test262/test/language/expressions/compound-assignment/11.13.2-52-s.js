// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-52-s
description: >
    Strict Mode - TypeError is thrown if The LeftHandSide of a
    Compound Assignment operator(>>>=) is a reference to a
    non-existent property of an object whose [[Extensible]] internal
    property if false
flags: [onlyStrict]
---*/

        var obj = {};
        Object.preventExtensions(obj);
assert.throws(TypeError, function() {
            obj.len >>>= 10;
});
