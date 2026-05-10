// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.2.3
description: FunctionPrototype `name` property
info: |
    The value of the name property of the Function prototype object is the
    empty String.

    ES6 Section 17:

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Function.prototype, "name", {
  value: "",
  writable: false,
  enumerable: false,
  configurable: true
});
