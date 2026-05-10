// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.11
description: RegExp.prototype[Symbol.split] `length` property
info: |
    The length property of the @@split method is 2.

    ES6 Section 17:

    [...]

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.split]
---*/

verifyProperty(RegExp.prototype[Symbol.split], "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true
});
