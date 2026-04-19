// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since when call is used for Function constructor themself new function instance creates
    and then first argument(thisArg) should be ignored
es5id: 15.3_A3_T2
description: First argument is string and null
---*/

this.color = "red";
var planet = "mars";

var f = Function.call("blablastring", "return this.color;");

assert.sameValue(f(), "red", 'f() must return "red"');

var g = Function.call(null, "return this.planet;");

assert.sameValue(g(), "mars", 'g() must return "mars"');
