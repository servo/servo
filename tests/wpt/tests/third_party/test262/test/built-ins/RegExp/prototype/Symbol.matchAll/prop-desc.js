// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.prototype[Symbol.matchAll] property descriptor
info: |
  17 ECMAScript Standard Built-in Objects:

    [...]

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Symbol.matchAll]
---*/

assert.sameValue(typeof RegExp.prototype[Symbol.matchAll], 'function');

verifyProperty(RegExp.prototype, Symbol.matchAll, {
  writable: true,
  enumerable: false,
  configurable: true,
});
