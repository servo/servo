// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-17gs
description: >
    Strict - checking 'this' from a global scope (eval used within
    strict mode)
flags: [onlyStrict]
---*/

if (eval("this") !== this) {
    throw "'this' had incorrect value!";
}
