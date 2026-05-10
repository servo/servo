// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: >
    The value of the name property of this function is "[Symbol.hasInstance]".

    17 ECMAScript Standard Built-in Objects
features: [Symbol.hasInstance]
includes: [propertyHelper.js]
---*/

verifyProperty(Function.prototype[Symbol.hasInstance], "name", {
  value: "[Symbol.hasInstance]",
  writable: false,
  enumerable: false,
  configurable: true
});
