// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-11-s
description: >
    Strict Mode - checking 'this' (Anonymous FunctionExpression
    defined within strict mode)
flags: [onlyStrict]
---*/

assert.sameValue((function () {
    return typeof this;
})(), "undefined");
