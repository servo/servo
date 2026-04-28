// Copyright (C) 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.2.14
description: >
    `Symbol.unscopables` property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Symbol.unscopables]
---*/

assert.sameValue(typeof Symbol.unscopables, 'symbol');
verifyProperty(Symbol, 'unscopables', {
  writable: false,
  enumerable: false,
  configurable: false,
});
