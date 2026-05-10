// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-regexp.prototype.dotall
description: >
  `pending` property descriptor
info: |
  RegExp.prototype.dotAll is an accessor property whose set accessor
  function is undefined.

  17 ECMAScript Standard Built-in Objects

  Every accessor property described in clauses 18 through 26 and in Annex B.2 has the attributes
  { [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified. If only a get
  accessor function is described, the set accessor function is the default value, undefined. If
  only a set accessor is described the get accessor is the default value, undefined.
includes: [propertyHelper.js]
features: [regexp-dotall]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, "dotAll");

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, "function");

verifyProperty(RegExp.prototype, "dotAll", {
  enumerable: false,
  configurable: true,
});
