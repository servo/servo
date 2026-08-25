// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flatmap
description: Property type and descriptor.
info: >
  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Array.prototype.flatMap]
---*/

assert.sameValue(
  typeof Array.prototype.flatMap,
  'function',
  '`typeof Array.prototype.flatMap` is `function`'
);

verifyProperty(Array.prototype, 'flatMap', {
  enumerable: false,
  writable: true,
  configurable: true,
});
