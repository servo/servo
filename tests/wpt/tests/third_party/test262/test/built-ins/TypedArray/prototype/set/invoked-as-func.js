// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set
description: Throws a TypeError exception when invoked as a function
info: |
  22.2.3.22 %TypedArray%.prototype.set ( overloaded [ , offset ])

  This function is not generic. The this value must be an object with a
  [[TypedArrayName]] internal slot.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var set = TypedArray.prototype.set;

assert.sameValue(typeof set, 'function');

assert.throws(TypeError, function() {
  set();
});
