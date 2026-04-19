// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-27gs
description: >
    Strict - checking 'this' from a global scope (FunctionDeclaration
    defined within a FunctionDeclaration inside strict mode)
flags: [onlyStrict]
---*/

function f1() {
    function f() {
        return typeof this;
    }
    return (f()==="undefined") && ((typeof this)==="undefined");
}
if (! f1()) {
    throw "'this' had incorrect value!";
}
