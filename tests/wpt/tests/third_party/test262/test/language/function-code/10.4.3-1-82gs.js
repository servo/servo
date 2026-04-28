// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-82gs
description: >
    Strict - checking 'this' from a global scope (non-strict function
    declaration called by strict eval)
flags: [noStrict]
---*/

function f() { return this!==undefined;};
if (! ((function () {"use strict"; return eval("f();");})()) ){
    throw "'this' had incorrect value!";
}
