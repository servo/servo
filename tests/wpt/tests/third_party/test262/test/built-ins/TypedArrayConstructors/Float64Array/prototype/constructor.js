// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.prototype.constructor
description: >
  The initial value of Float64Array.prototype.constructor is the Float64Array object.
info: |
  The initial value of Float64Array.prototype.constructor is the intrinsic
  object %Float64Array%.

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in Annex B.2 has
    the attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }
    unless otherwise specified.
includes: [propertyHelper.js]
features: [TypedArray]
---*/

assert.sameValue(Float64Array.prototype.constructor, Float64Array);

verifyNotEnumerable(Float64Array.prototype, "constructor");
verifyWritable(Float64Array.prototype, "constructor");
verifyConfigurable(Float64Array.prototype, "constructor");
