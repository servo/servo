// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-15gs
description: >
    Strict - checking 'this' from a global scope (New'ed Function
    constructor defined within strict mode)
flags: [onlyStrict]
---*/

var f = new Function("return typeof this;");
if (f() === "undefined") {
    throw "'this' had incorrect value!";
}
