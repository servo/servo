// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-reflect-object
description: >
  Property descriptor of Reflect
info: |
  The Reflect Object

  ...
  The Reflect object does not have a [[Construct]] internal method;
  it is not possible to use the Reflect object as a constructor with the new operator.

  The Reflect object does not have a [[Call]] internal method;
  it is not possible to invoke the Reflect object as a function.

  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Reflect]
---*/

assert.sameValue(typeof Reflect, "object");

assert.throws(TypeError, function() {
  Reflect();
}, "no [[Call]]");

assert.throws(TypeError, function() {
  new Reflect();
}, "no [[Construct]]");

verifyProperty(this, "Reflect", {
  enumerable: false,
  writable: true,
  configurable: true
});
