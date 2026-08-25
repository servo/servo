// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json-object
description: >
  Property descriptor of JSON
info: |
  The JSON Object

  ...
  The JSON object does not have a [[Construct]] internal method;
  it is not possible to use the JSON object as a constructor with the new operator.

  The JSON object does not have a [[Call]] internal method;
  it is not possible to invoke the JSON object as a function.

  17 ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(typeof JSON, "object");

assert.throws(TypeError, function() {
  JSON();
}, "no [[Call]]");

assert.throws(TypeError, function() {
  new JSON();
}, "no [[Construct]]");

verifyProperty(this, "JSON", {
  enumerable: false,
  writable: true,
  configurable: true
});
