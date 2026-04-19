// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
  Non-writable properties are overwritten by CreateDataPropertyOrThrow.
info: |
  22.1.3.7 Array.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  8. Repeat, while k < len
    ...
    c. If kPresent is true, then
      ...
      iii. If selected is true, then
        1. Perform ? CreateDataPropertyOrThrow(A, ! ToString(to), kValue).
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

var r = a.filter(function(){ return true; });

verifyProperty(r, 0, {
    value: 1, writable: true, configurable: true, enumerable: true,
});
