// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-77gs
description: >
    checking 'this' from a global scope (strict function declaration called by
    Function.prototype.bind(null)())
---*/

function f() { "use strict"; return this===null;};
if (! (f.bind(null)())){
    throw "'this' had incorrect value!";
}
