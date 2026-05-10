// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
  %RegExpStringIteratorPrototype%.next property descriptor
info: |
  17 ECMAScript Standard Built-in Objects:

    [...]

    Every other data property described in clauses 18 through 26 and in Annex
    B.2 has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Symbol.matchAll]
---*/

var RegExpStringIteratorProto = Object.getPrototypeOf(/./[Symbol.matchAll](''));

assert.sameValue(typeof RegExpStringIteratorProto.next, 'function');

verifyProperty(RegExpStringIteratorProto, 'next', {
  writable: true,
  enumerable: false,
  configurable: true
});
