// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype
description: >
  The property descriptor of Array.prototype
info: |
  22.1.2.4 Array.prototype

  The value of Array.prototype is %ArrayPrototype%, the intrinsic Array prototype object.

  This property has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: false }.
includes: [propertyHelper.js]
---*/

verifyProperty(Array, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false,
});
