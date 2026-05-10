// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.9
description: RegExp.prototype[Symbol.search] `name` property
info: |
    The value of the name property of this function is "[Symbol.search]".

    ES6 Section 17:

    [...]

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
features: [Symbol.search]
includes: [propertyHelper.js]
---*/

verifyProperty(RegExp.prototype[Symbol.search], "name", {
  value: "[Symbol.search]",
  writable: false,
  enumerable: false,
  configurable: true
});
