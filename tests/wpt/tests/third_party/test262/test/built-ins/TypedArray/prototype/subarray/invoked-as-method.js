// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Requires a [[TypedArrayName]] internal slot.
info: |
  22.2.3.26 %TypedArray%.prototype.subarray( [ begin [ , end ] ] )

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.sameValue(typeof TypedArrayPrototype.subarray, 'function');

assert.throws(TypeError, function() {
  TypedArrayPrototype.subarray();
});
