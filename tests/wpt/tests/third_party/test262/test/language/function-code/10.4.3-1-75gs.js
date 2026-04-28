// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-75gs
description: >
    checking 'this' from a global scope (strict function declaration called by
    Function.prototype.call(globalObject))
---*/

function f() { "use strict"; return this;};
if (f.call(this) !== this){
    throw "'this' had incorrect value!";
}
