// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-23gs
description: >
    Strict - checking 'this' from a global scope (New'ed object from
    FunctionExpression defined within strict mode)
flags: [onlyStrict]
---*/

var f = function () {
    return this;
}
if (((new f()) === this) || (typeof (new f()) === "undefined")) {
    throw "'this' had incorrect value!";
}
