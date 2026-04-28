// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The call method performs a function call using the [[Call]] property of
    the object. If the object does not have a [[Call]] property, a TypeError
    exception is thrown
es5id: 15.3.4.4_A1_T2
description: >
    Calling "call" method of the object that does not have a [[Call]]
    property.  Prototype of the object is Function.prototype
---*/

function FACTORY() {}

FACTORY.prototype = Function.prototype;

var obj = new FACTORY;

assert.sameValue(typeof obj.call, "function", 'The value of `typeof obj.call` is expected to be "function"');

try {
  obj.call();
  throw new Test262Error('#2: If the object does not have a [[Call]] property, a TypeError exception is thrown');
} catch (e) {
  assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
}
