// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math-object
description: >
  Property descriptor of Math
info: |
  The Math Object

  ...
  The Math object does not have a [[Construct]] internal method;
  it is not possible to use the Math object as a constructor with the new operator.

  The Math object does not have a [[Call]] internal method;
  it is not possible to invoke the Math object as a function.

  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof Math, "object");

assert.throws(TypeError, function() {
  Math();
}, "no [[Call]]");

assert.throws(TypeError, function() {
  new Math();
}, "no [[Construct]]");

verifyProperty(this, "Math", {
  enumerable: false,
  writable: true,
  configurable: true
});
