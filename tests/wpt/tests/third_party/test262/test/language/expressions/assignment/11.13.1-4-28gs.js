// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.13.1-4-28gs
description: >
    Strict Mode - TypeError is thrown if the identifier 'Math.PI'
    appears as the LeftHandSideExpression of simple assignment(=)
flags: [onlyStrict]
---*/

assert.throws(TypeError, function() {
  Math.PI = 20;
});
