// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-42gs
description: >
    Strict - checking 'this' from a global scope (FunctionDeclaration
    defined within an Anonymous FunctionExpression with a strict
    directive prologue)
flags: [noStrict]
---*/

if (! ((function () {
    "use strict";
    function f() {
        return typeof this;
    }
    return (f()==="undefined") && ((typeof this)==="undefined");
})())) {
    throw "'this' had incorrect value!";
}
