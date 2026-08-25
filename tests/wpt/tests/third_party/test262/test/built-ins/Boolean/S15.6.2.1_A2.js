// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object
    is set to the original Boolean prototype object, the one that is the
    initial value of Boolean.prototype
esid: sec-boolean-constructor
description: Checking prototype property of the newly created object
---*/

// CHECK#1
var x1 = new Boolean(1);

assert.sameValue(
  typeof x1.constructor.prototype,
  "object",
  'The value of `typeof x1.constructor.prototype` is expected to be "object"'
);

var x2 = new Boolean(2);
assert(Boolean.prototype.isPrototypeOf(x2), 'Boolean.prototype.isPrototypeOf(x2) must return true');

var x3 = new Boolean(3);

assert.sameValue(
  Boolean.prototype,
  x3.constructor.prototype,
  'The value of Boolean.prototype is expected to equal the value of x3.constructor.prototype'
);
