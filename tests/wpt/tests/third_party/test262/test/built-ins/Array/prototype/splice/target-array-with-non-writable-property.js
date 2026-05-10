// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  Non-writable properties are overwritten by CreateDataPropertyOrThrow.
info: |
  22.1.3.26 Array.prototype.splice ( start, deleteCount, ...items )

  ...
  11. Repeat, while k < actualDeleteCount
    ...
    c. If fromPresent is true, then
      ...
      ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(k), fromValue).
    ...
features: [Symbol.species]
includes: [propertyHelper.js]
---*/

var a = [1];
a.constructor = {};
a.constructor[Symbol.species] = function(len) {
    var q = new Array(0);
    Object.defineProperty(q, 0, {
        value: 0, writable: false, configurable: true, enumerable: false,
    });
    return q;
};

var r = a.splice(0);

verifyProperty(r, 0, {
    value: 1, writable: true, configurable: true, enumerable: true,
});
