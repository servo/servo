// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-10-s
description: >
    StrictMode - writing a property named 'caller' of function objects
    is not allowed outside the function
---*/

var foo = Function("'use strict';");

assert.throws(TypeError, function() {
    foo.caller = 41;
});
