// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-53gs
description: >
    Strict - checking 'this' from a global scope (Anonymous
    FunctionExpression with a strict directive prologue defined within
    an Anonymous FunctionExpression)
flags: [noStrict]
---*/

var global = this;

if (! ((function () {
    return ((function () {
        "use strict";
        return typeof this;
    })()==="undefined") && (this===global);
})())) {
    throw "'this' had incorrect value!";
}
