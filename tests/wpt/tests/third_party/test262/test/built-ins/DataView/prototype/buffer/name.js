// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-dataview.prototype.buffer
description: >
  get DataView.prototype.buffer

  17 ECMAScript Standard Built-in Objects

  Functions that are specified as get or set accessor functions of built-in
  properties have "get " or "set " prepended to the property name string.

includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(
  DataView.prototype, 'buffer'
);

verifyProperty(descriptor.get, "name", {
  value: "get buffer",
  writable: false,
  enumerable: false,
  configurable: true
});
