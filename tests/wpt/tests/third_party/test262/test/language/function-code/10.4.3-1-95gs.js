// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-95gs
description: >
    Strict - checking 'this' from a global scope (non-strict function
    declaration called by strict Function.prototype.bind()())
flags: [noStrict]
---*/

var global = this;
function f() { return this===global;};
if (! ((function () {"use strict"; return f.bind()(); })())){
    throw "'this' had incorrect value!";
}
