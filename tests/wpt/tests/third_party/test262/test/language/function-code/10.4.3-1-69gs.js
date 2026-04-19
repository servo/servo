// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-69gs
description: >
    checking 'this' from a global scope (strict function declaration called by
    Function.prototype.apply(someObject))
---*/

var o = {};
function f() { "use strict"; return this===o;};
if (! f.apply(o)){
    throw "'this' had incorrect value!";
}
