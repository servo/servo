// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%asynciteratorprototype%-@@asyncDispose
description: Property descriptor of %AsyncIteratorPrototype%[ @@asyncDispose ]
info: |
    ES6 Section 17

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
features: [explicit-resource-management]
includes: [propertyHelper.js]
---*/

async function* generator() {}
const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))

verifyProperty(AsyncIteratorPrototype, Symbol.asyncDispose, {
  writable: true,
  enumerable: false,
  configurable: true,
});
