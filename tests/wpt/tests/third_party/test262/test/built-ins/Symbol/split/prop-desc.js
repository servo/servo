// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.2.11
description: >
    `Symbol.split` property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Symbol.split]
---*/

assert.sameValue(typeof Symbol.split, 'symbol');
verifyProperty(Symbol, 'split', {
  writable: false,
  enumerable: false,
  configurable: false,
});
