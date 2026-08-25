// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.constructor
description: >
  Array.prototype.constructor
info: |
  22.1.3.2 Array.prototype.constructor

  The initial value of Array.prototype.constructor is the intrinsic object %Array%.

  17 ECMAScript Standard Built-in Objects
  
  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
    [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(Array.prototype.constructor, Array);

verifyProperty(Array.prototype, "constructor", {
  writable: true,
  enumerable: false,
  configurable: true
});
