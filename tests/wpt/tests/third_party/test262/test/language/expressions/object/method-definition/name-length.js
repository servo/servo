// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions declared as methods have a `length` property that describes the
    number of formal parameters.
es6id: 14.3.8
includes: [propertyHelper.js]
---*/

var method = { method(a, b, c) {} }.method;

verifyProperty(method, "length", {
  value: 3,
  writable: false,
  enumerable: false,
  configurable: true,
});
