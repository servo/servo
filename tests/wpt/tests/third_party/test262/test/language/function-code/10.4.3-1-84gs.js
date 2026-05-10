// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-84gs
description: >
    Strict - checking 'this' from a global scope (non-strict function
    declaration called by strict new'ed Function constructor)
flags: [noStrict]
---*/

function f() { return this!==undefined;};
if (! ((function () {return new Function("\"use strict\";return f();")();})()) ){
    throw "'this' had incorrect value!";
}
