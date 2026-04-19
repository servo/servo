// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set.prototype-@@tostringtag
description: >
    `Object.prototype.getOwnPropertyDescriptor` should reflect the value and
    writability of the @@toStringTag attribute.
includes: [propertyHelper.js]
features: [Symbol.toStringTag]
---*/

var SetProto = Object.getPrototypeOf(new Set());

assert.sameValue(
  SetProto[Symbol.toStringTag],
  'Set',
  "The value of `SetProto[Symbol.toStringTag]` is `'Set'`"
);

verifyProperty(SetProto, Symbol.toStringTag, {
  writable: false,
  enumerable: false,
  configurable: true,
});
