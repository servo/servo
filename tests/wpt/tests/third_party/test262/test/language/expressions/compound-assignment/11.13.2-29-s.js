// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.2-29-s
description: >
    Strict Mode - TypeError is thrown if The LeftHandSide of a
    Compound Assignment operator(>>=) is a reference to a data
    property with the attribute value {[[Writable]]:false}
flags: [onlyStrict]
---*/

        var obj = {};
        Object.defineProperty(obj, "prop", {
            value: 10,
            writable: false,
            enumerable: true,
            configurable: true
        });
assert.throws(TypeError, function() {
            obj.prop >>= 20;
});
assert.sameValue(obj.prop, 10, 'obj.prop');
