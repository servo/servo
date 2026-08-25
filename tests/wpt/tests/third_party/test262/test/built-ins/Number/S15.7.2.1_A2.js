// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The [[Prototype]] property of the newly constructed object
    is set to the original Number prototype object, the one that is the
    initial value of Number.prototype
es5id: 15.7.2.1_A2
description: Checking prototype property of the newly created objects
---*/

// CHECK#1
var x1 = new Number(1);

assert.sameValue(
  typeof x1.constructor.prototype,
  "object",
  'The value of `typeof x1.constructor.prototype` is expected to be "object"'
);

var x2 = new Number(2);
assert(Number.prototype.isPrototypeOf(x2), 'Number.prototype.isPrototypeOf(x2) must return true');

var x3 = new Number(3);

assert.sameValue(
  Number.prototype,
  x3.constructor.prototype,
  'The value of Number.prototype is expected to equal the value of x3.constructor.prototype'
);
