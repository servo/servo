// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2-13-s
description: >
    StrictMode - reading a property named 'arguments' of function
    objects is not allowed outside the function
---*/

var foo = new Function("'use strict';");

assert.throws(TypeError, function() {
    var temp = foo.arguments;
});
