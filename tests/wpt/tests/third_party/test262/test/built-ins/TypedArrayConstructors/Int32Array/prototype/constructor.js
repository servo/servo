// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray.prototype.constructor
description: >
  The initial value of Int32Array.prototype.constructor is the Int32Array object.
info: |
  The initial value of Int32Array.prototype.constructor is the intrinsic
  object %Int32Array%.

  17 ECMAScript Standard Built-in Objects:
    Every other data property described in clauses 18 through 26 and in Annex B.2 has
    the attributes { [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }
    unless otherwise specified.
includes: [propertyHelper.js]
features: [TypedArray]
---*/

assert.sameValue(Int32Array.prototype.constructor, Int32Array);

verifyNotEnumerable(Int32Array.prototype, "constructor");
verifyWritable(Int32Array.prototype, "constructor");
verifyConfigurable(Int32Array.prototype, "constructor");
