// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since when call is used for Function constructor themself new function instance creates
    and then first argument(thisArg) should be ignored
es5id: 15.3_A3_T5
description: >
    First argument is this, and this don`t have needed variable.
    Function return this.var_name
---*/

var f = Function.call(this, "return this.planet;");
var g = Function.call(this, "return this.color;");

assert.sameValue(f(), undefined, 'f() returns undefined');

var planet = "mars";

assert.sameValue(f(), "mars", 'f() must return "mars"');
assert.sameValue(g(), undefined, 'g() returns undefined');

this.color = "red";

assert.sameValue(g(), "red", 'g() must return "red"');
