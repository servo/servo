// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.6
description: RegExp.prototype[Symbol.match] `name` property
info: |
    The value of the name property of this function is "[Symbol.match]".

    ES6 Section 17:

    [...]

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.match]
---*/

verifyProperty(RegExp.prototype[Symbol.match], "name", {
  value: "[Symbol.match]",
  writable: false,
  enumerable: false,
  configurable: true
});
