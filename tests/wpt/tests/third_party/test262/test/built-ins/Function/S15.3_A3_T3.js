// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since when call is used for Function constructor themself new function instance creates
    and then first argument(thisArg) should be ignored
es5id: 15.3_A3_T3
description: First argument is this, and this don`t have needed variable
---*/

var f = Function.call(this, "return planet;");
var g = Function.call(this, "return color;");

assert.sameValue(f(), undefined, 'f() returns undefined');

var planet = "mars";

assert.sameValue(f(), "mars", 'f() must return "mars"');

try {
  g();
  throw new Test262Error('#3: ');
} catch (e) {
  assert(
    e instanceof ReferenceError,
    'The result of evaluating (e instanceof ReferenceError) is expected to be true'
  );
}

this.color = "red";

assert.sameValue(g(), "red", 'g() must return "red"');
