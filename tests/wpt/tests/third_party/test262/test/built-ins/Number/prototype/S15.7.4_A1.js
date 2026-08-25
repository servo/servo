// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Number prototype object is itself a Number object
    (its [[Class]] is "Number") whose value is +0
es5id: 15.7.4_A1
description: Checking type and value of Number.prototype property
---*/
assert.sameValue(
  typeof Number.prototype,
  "object",
  'The value of `typeof Number.prototype` is expected to be "object"'
);

assert(Number.prototype == 0, 'The value of Number.prototype is expected to be 0');

delete Number.prototype.toString;

assert.sameValue(
  Number.prototype.toString(),
  "[object Number]",
  'Number.prototype.toString() must return "[object Number]"'
);
