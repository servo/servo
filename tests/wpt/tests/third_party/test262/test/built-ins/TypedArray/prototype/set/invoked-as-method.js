// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set
description: Requires a [[TypedArrayName]] internal slot.
info: |
  22.2.3.22 %TypedArray%.prototype.set ( overloaded [ , offset ])

  This function is not generic. The this value must be an object with a
  [[TypedArrayName]] internal slot.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.sameValue(typeof TypedArrayPrototype.set, 'function');

assert.throws(TypeError, function() {
  TypedArrayPrototype.set();
});
