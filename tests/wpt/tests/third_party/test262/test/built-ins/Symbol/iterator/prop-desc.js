// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.2.4
description: >
    `Symbol.iterator` property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Symbol.iterator]
---*/

assert.sameValue(typeof Symbol.iterator, 'symbol');
verifyProperty(Symbol, 'iterator', {
  writable: false,
  enumerable: false,
  configurable: false,
});
