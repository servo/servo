// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
  Non-writable properties are overwritten by CreateDataPropertyOrThrow.
info: |
  22.1.3.16 Array.prototype.map ( callbackfn [ , thisArg ] )

  ...
  7. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      iii. Perform ? CreateDataPropertyOrThrow(A, Pk, mappedValue).
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

var r = a.map(function(){ return 2; });

verifyProperty(r, 0, {
    value: 2, writable: true, configurable: true, enumerable: true,
});
