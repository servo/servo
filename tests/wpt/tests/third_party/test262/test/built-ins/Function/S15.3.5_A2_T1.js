// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Every function instance has a [[Call]] property
es5id: 15.3.5_A2_T1
description: For testing call Function("var x =1; this.y=2;return \"OK\";")()
---*/
assert.sameValue(
  Function("var x =1; this.y=2;return \"OK\";")(),
  "OK",
  'Function("var x =1; this.y=2;return "OK";")() must return "OK"'
);

assert.sameValue(typeof x, "undefined", 'The value of `typeof x` is expected to be "undefined"');
assert.sameValue(y, 2, 'The value of y is expected to be 2');
