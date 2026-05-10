// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-10gs
description: >
    Strict - checking 'this' from a global scope (FunctionExpression
    includes strict directive prologue)
flags: [noStrict]
---*/

var f = function () {
    "use strict";
    return typeof this;
}
if (f() !== "undefined") {
    throw "'this' had incorrect value!";
}
