// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.5.6.3.1
description: >
  The initial value of SyntaxError.prototype.constructor is the SyntaxError object.
info: |
  The initial value of the constructor property of the prototype for a given NativeError
  constructor is the corresponding intrinsic object %NativeError% (19.5.6.1).

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in Annex B.2 has
    the attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }
    unless otherwise specified.
includes: [propertyHelper.js]
---*/

assert.sameValue(SyntaxError.prototype.constructor, SyntaxError);

verifyProperty(SyntaxError.prototype, "constructor", {
  writable: true,
  enumerable: false,
  configurable: true,
});
