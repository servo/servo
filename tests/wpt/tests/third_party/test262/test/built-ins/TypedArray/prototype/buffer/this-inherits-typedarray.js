// Copyright (C) 2020 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.buffer
description: >
  Throws a TypeError exception when `this` does not have a [[TypedArrayName]]
  internal slot, even if its prototype does
info: |
  22.2.3.1 get %TypedArray%.prototype.buffer

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  3. If O does not have a [[TypedArrayName]] internal slot, throw a TypeError
  exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;
var getter = Object.getOwnPropertyDescriptor(
  TypedArrayPrototype, "buffer"
).get;

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var typedArray = new TA(makeCtorArg(5));
  var o = {};
  Object.setPrototypeOf(o, typedArray);
  assert.throws(TypeError, function() {
    getter.call(o);
  },
  "Internal slot should not be inherited");
});
