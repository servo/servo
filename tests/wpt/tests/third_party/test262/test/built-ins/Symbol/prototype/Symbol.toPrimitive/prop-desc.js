// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.4.3.4
description: Symbol.prototype[Symbol.toPrimitive] property descriptor
info: |
    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.toPrimitive]
---*/

verifyProperty(Symbol.prototype, Symbol.toPrimitive, {
  writable: false,
  enumerable: false,
  configurable: true,
});
