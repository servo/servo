// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-20gs
description: >
    Strict - checking 'this' from a global scope (indirect eval
    includes strict directive prologue)
flags: [noStrict]
---*/

var my_eval = eval;
if (my_eval("\"use strict\";\nthis") !== this ) {
    throw "'this' had incorrect value!";
}
