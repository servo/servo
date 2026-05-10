// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
    `Symbol.toStringTag` property descriptor
info: |
    The initial value of the @@toStringTag property is the string value "String
    Iterator".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
features: [Symbol.matchAll, Symbol.toStringTag]
includes: [propertyHelper.js]
---*/

var RegExpStringIteratorProto = Object.getPrototypeOf(/./[Symbol.matchAll](''));

assert.sameValue(RegExpStringIteratorProto[Symbol.toStringTag], 'RegExp String Iterator');

verifyProperty(RegExpStringIteratorProto, Symbol.toStringTag, {
  writable: false,
  enumerable: false,
  configurable: true,
});
