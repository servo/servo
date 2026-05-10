// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asynciteratorprototype
description: Property descriptor
info: |
    ECMAScript Standard Built-in Objects

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
features: [Symbol.asyncIterator, async-iteration]
includes: [propertyHelper.js]
---*/

async function* generator() {}
var AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))

assert.sameValue(typeof AsyncIteratorPrototype[Symbol.asyncIterator], 'function');

verifyProperty(AsyncIteratorPrototype, Symbol.asyncIterator, {
  enumerable: false,
  writable: true,
  configurable: true,
});

