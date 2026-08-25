// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.4
description: >
  RegExp.prototype.global name
info: |
  17 ECMAScript Standard Built-in Objects

  Functions that are specified as get or set accessor functions of built-in
  properties have "get " or "set " prepended to the property name string.
includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(RegExp.prototype, 'global');

verifyProperty(descriptor.get, "name", {
  value: "get global",
  writable: false,
  enumerable: false,
  configurable: true
});
