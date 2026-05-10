// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generator functions declared as methods have a `length` property that
    describes the number of formal parameters.
es6id: 14.4.13
includes: [propertyHelper.js]
features: [generators]
---*/

var method = { *method(a, b, c) {} }.method;

verifyProperty(method, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true,
});
