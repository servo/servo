// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-45gs
description: >
    Strict - checking 'this' from a global scope (FunctionDeclaration
    with a strict directive prologue defined within a
    FunctionDeclaration)
flags: [noStrict]
---*/

var global = this;

function f1() {
    function f() {
        "use strict";
        return typeof this;
    }
    return (f()==="undefined") && (this===global);
}
if (! f1()) {
    throw "'this' had incorrect value!";
}
