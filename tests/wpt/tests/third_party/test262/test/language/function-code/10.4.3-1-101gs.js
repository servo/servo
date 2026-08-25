// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-101gs
description: >
    Strict Mode - checking 'this' (non-strict function passed as arg
    to String.prototype.replace from strict context)
flags: [noStrict]
---*/

var x = 3;

function f() {
    x = this;
    return "a";
}

if ( (!(function() {"use strict"; return "ab".replace("b", f)==="aa";}())) || (x!==this)) {
    throw "'this' had incorrect value!";
}
