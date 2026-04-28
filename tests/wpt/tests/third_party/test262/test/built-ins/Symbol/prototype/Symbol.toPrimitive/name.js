// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.3.4
description: Symbol.prototype[Symbol.toPrimitive] `name` property
info: |
    The value of the name property of this function is "[Symbol.toPrimitive]".

    ES6 Section 17:

    [...]

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
features: [Symbol.toPrimitive]
includes: [propertyHelper.js]
---*/

verifyProperty(Symbol.prototype[Symbol.toPrimitive], "name", {
  value: "[Symbol.toPrimitive]",
  writable: false,
  enumerable: false,
  configurable: true
});
