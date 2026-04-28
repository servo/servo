// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tostring
description: >
  "toString" property of TypedArrayPrototype
info: |
  22.2.3.28 %TypedArray%.prototype.toString ( )

  The initial value of the %TypedArray%.prototype.toString data property is the
  same built-in function object as the Array.prototype.toString method defined
  in 22.1.3.27.

  ES6 section 17: Every other data property described in clauses 18 through 26
  and in Annex B.2 has the attributes { [[Writable]]: true,
  [[Enumerable]]: false, [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js, testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.sameValue(TypedArrayPrototype.toString, Array.prototype.toString);

verifyProperty(TypedArrayPrototype, 'toString', {
  writable: true,
  enumerable: false,
  configurable: true
});
