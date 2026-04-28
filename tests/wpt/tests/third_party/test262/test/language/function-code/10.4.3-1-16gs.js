// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-16gs
description: >
    Strict - checking 'this' from a global scope (New'ed Function
    constructor includes strict directive prologue)
flags: [noStrict]
---*/

var f = new Function("\"use strict\";\nreturn typeof this;");
if (f() !== "undefined") {
    throw "'this' had incorrect value!";
}
