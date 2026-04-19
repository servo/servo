// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-symbol.asynciterator
description: >
    `Symbol.asyncIterator` property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Symbol.asyncIterator]
---*/

assert.sameValue(typeof Symbol.asyncIterator, 'symbol');

verifyProperty(Symbol, 'asyncIterator', {
    enumerable: false,
    writable: false,
    configurable: false,
});
