// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-25gs
description: >
    Strict - checking 'this' from a global scope (New'ed object from
    Anonymous FunctionExpression defined within strict mode)
flags: [onlyStrict]
---*/

var obj = new (function () {
    return this;
});
if ((obj === this) || (typeof obj === "undefined")) {
    throw "'this' had incorrect value!";
}
