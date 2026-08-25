// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-set.prototype.size
description: >
    get Set.prototype.size

    17 ECMAScript Standard Built-in Objects

    Functions that are specified as get or set accessor functions of built-in
    properties have "get " or "set " prepended to the property name string.

includes: [propertyHelper.js]
---*/

var descriptor = Object.getOwnPropertyDescriptor(Set.prototype, "size");


verifyProperty(descriptor.get, "name", {
  value: "get size",
  writable: false,
  enumerable: false,
  configurable: true
});
