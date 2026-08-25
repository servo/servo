// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.2.3
description: Property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Symbol.isConcatSpreadable]
---*/

assert.sameValue(typeof Symbol.isConcatSpreadable, 'symbol');
verifyProperty(Symbol, 'isConcatSpreadable', {
  writable: false,
  enumerable: false,
  configurable: false,
});
