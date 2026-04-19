// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-50gs
description: >
    Strict - checking 'this' from a global scope (Anonymous
    FunctionExpression with a strict directive prologue defined within
    a FunctionExpression)
flags: [noStrict]
---*/

var global = this;

var f1 = function () {
    return ((function () {
        "use strict";
        return typeof this;
    })()==="undefined") && (this===global);
}
if (! f1()) {
    throw "'this' had incorrect value!";
}
