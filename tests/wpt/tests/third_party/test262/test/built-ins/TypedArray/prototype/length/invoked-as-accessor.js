// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.length
description: >
  Requires this value to have a [[ViewedArrayBuffer]] internal slot
info: |
  22.2.3.17 get %TypedArray%.prototype.length

  1. Let O be the this value.
  ...
  3. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.throws(TypeError, function() {
  TypedArrayPrototype.length;
});
