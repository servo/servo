// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Every function instance has a [[Call]] property
es5id: 15.3.5_A2_T2
description: >
    For testing call (new Function("arg1,arg2","var x =arg1;
    this.y=arg2;return arg1+arg2;"))("1",2)
---*/
assert.sameValue(
  (new Function("arg1,arg2", "var x =arg1; this.y=arg2;return arg1+arg2;"))("1", 2),
  "12",
  'new Function("arg1,arg2", "var x =arg1; this.y=arg2;return arg1+arg2;")(1, 2) must return "12"'
);

assert.sameValue(typeof x, "undefined", 'The value of `typeof x` is expected to be "undefined"');
assert.sameValue(y, 2, 'The value of y is expected to be 2');
