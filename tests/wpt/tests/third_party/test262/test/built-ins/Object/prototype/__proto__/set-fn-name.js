// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: B.2.2.1.2
description: >
    set Object.prototype.__proto__

    17 ECMAScript Standard Built-in Objects

    Functions that are specified as get or set accessor functions of built-in
    properties have "get " or "set " prepended to the property name string.

includes: [propertyHelper.js]
features: [__proto__]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__');

verifyProperty(descriptor.set, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: "set __proto__"
});
