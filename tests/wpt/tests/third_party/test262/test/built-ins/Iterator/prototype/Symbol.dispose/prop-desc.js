// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%iteratorprototype%-@@dispose
description: Property descriptor of %IteratorPrototype%[ @@dispose ]
info: |
    ES6 Section 17

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
features: [explicit-resource-management]
includes: [propertyHelper.js]
---*/
const IteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

verifyProperty(IteratorPrototype, Symbol.dispose, {
  writable: true,
  enumerable: false,
  configurable: true,
});
