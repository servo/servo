// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-62gs
description: >
    checking 'this' from a global scope (strict function declaration called by
    non-strict function declaration)
---*/

function f() { "use strict"; return this;};
function foo() { return f();}
if (foo()!==undefined){
    throw "'this' had incorrect value!";
}
