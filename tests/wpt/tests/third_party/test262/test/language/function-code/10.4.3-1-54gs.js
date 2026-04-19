// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-54gs
description: >
    Strict - checking 'this' from a global scope (Literal getter
    defined within strict mode)
flags: [noStrict]
---*/

"use strict";
var o = { get foo() { return this; } }
if (o.foo!==o) {
    throw "'this' had incorrect value!";
}
