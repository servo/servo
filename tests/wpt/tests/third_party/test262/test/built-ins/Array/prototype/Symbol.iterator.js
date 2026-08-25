// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Initial state of the Symbol.iterator property
info: |
    The initial value of the @@iterator property is the same function object as
    the initial value of the Array.prototype.values property.

    Per ES6 section 17, the method should exist on the Array prototype, and it
    should be writable and configurable, but not enumerable.
includes: [propertyHelper.js]
features: [Symbol.iterator]
esid: sec-array.prototype-@@iterator
---*/

assert.sameValue(Array.prototype[Symbol.iterator], Array.prototype.values);

verifyProperty(Array.prototype, Symbol.iterator, {
  writable: true,
  enumerable: false,
  configurable: true
});
