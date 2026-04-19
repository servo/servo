// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: String.prototype.matchAll property descriptor
info: |
  17 ECMAScript Standard Built-in Objects:

    [...]

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [String.prototype.matchAll]
---*/

assert.sameValue(typeof String.prototype.matchAll, 'function');

verifyProperty(String.prototype, 'matchAll', {
  writable: true,
  enumerable: false,
  configurable: true,
});
