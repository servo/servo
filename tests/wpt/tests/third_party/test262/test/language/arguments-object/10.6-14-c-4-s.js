// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-14-c-4-s
description: >
    Strict Mode - TypeError is thrown when accessing the [[Set]]
    attribute in 'callee' under strict mode
flags: [onlyStrict]
---*/

var argObj = function () {
    return arguments;
} ();

assert.throws(TypeError, function() {
    argObj.callee = {};
});
