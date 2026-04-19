// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-43gs
description: >
    Strict - checking 'this' from a global scope (FunctionExpression
    defined within an Anonymous FunctionExpression with a strict
    directive prologue)
flags: [noStrict]
---*/

if (! ((function () {
    "use strict";
    var f = function () {
        return typeof this;
    }
    return (f()==="undefined") && ((typeof this)==="undefined");
})())) {
    throw "'this' had incorrect value!";
}
