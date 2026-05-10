// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.length
description: Throws a TypeError exception when invoked as a function
info: |
  22.2.3.17 get %TypedArray%.prototype.length

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;
var getter = Object.getOwnPropertyDescriptor(
  TypedArrayPrototype, 'length'
).get;

assert.throws(TypeError, function() {
  getter();
});
