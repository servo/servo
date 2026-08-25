// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.3-1-60gs
description: >
    checking 'this' from a global scope (Injected setter)
---*/

var o = {};
var x = 2;
Object.defineProperty(o, "foo", { set: function(stuff) { x=this; } });
o.foo = 3;
if (x!==o) {
    throw "'this' had incorrect value!";
}
