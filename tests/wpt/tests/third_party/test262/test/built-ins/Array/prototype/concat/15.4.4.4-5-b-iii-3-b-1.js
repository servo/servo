// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.concat
description: >
    Array.prototype.concat will concat an Array when index property
    (read-only) exists in Array.prototype (Step 5.b.iii.3.b)
includes: [propertyHelper.js]
---*/

Object.defineProperty(Array.prototype, "0", {
  value: 100,
  writable: false,
  configurable: true
});

var oldArr = [101];
var newArr = Array.prototype.concat.call(oldArr);

verifyProperty(newArr, "0", {
  value: 101,
  writable: true,
  enumerable: true,
  configurable: true,
});
