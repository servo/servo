// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: >
  Non-writable properties are overwritten by CreateDataPropertyOrThrow.
info: |
  22.1.3.23 Array.prototype.slice ( start, end )

  ...
  10. Repeat, while k < final
    ...
    c. If kPresent is true, then
      ...
      ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(n), kValue).
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

var r = a.slice(0);

verifyProperty(r, 0, {
    value: 1, writable: true, configurable: true, enumerable: true,
});
