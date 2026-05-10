// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-12-s
description: >
    Strict Mode - checking 'this' (Anonymous FunctionExpression
    includes strict directive prologue)
flags: [noStrict]
---*/

assert.sameValue((function () {
    "use strict";
    return typeof this;
})(), "undefined");
