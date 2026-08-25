// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-12gs
description: >
    Strict - checking 'this' from a global scope (Anonymous
    FunctionExpression includes strict directive prologue)
flags: [noStrict]
---*/

if ((function () {
    "use strict";
    return typeof this;
})() !== "undefined") {
    throw "'this' had incorrect value!";
}
