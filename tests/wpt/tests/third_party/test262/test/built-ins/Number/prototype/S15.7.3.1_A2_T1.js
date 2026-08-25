// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number.prototype is itself Number object
es5id: 15.7.3.1_A2_T1
description: >
    Checking type of Number.prototype property - test based on
    deleting Number.prototype.toString
---*/
assert.sameValue(
  typeof Number.prototype,
  "object",
  'The value of `typeof Number.prototype` is expected to be "object"'
);

delete Number.prototype.toString;

assert.sameValue(
  Number.prototype.toString(),
  "[object Number]",
  'Number.prototype.toString() must return "[object Number]"'
);
