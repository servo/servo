// Copyright (C) 2017 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint-constructor
description: >
  Property descriptor of BigInt
info: |
  The BigInt Object

  ECMAScript Standard Built-in Objects:

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(this, "BigInt", {
  enumerable: false,
  writable: true,
  configurable: true
});
