// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-81gs
description: >
    Strict - checking 'this' from a global scope (non-strict function
    declaration called by strict function declaration)
flags: [noStrict]
---*/

function f() { return this!==undefined;};
function foo() { "use strict"; return f();}
if (! foo()){
    throw "'this' had incorrect value!";
}
