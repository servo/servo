// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: Return -1 if fromIndex >= ArrayLength - converted values
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  4. Let n be ? ToInteger(fromIndex). (If fromIndex is undefined, this step
  produces the value 0.)
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var obj = {
  valueOf: function() {
    return 1;
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([42n, 43n]));
  assert.sameValue(sample.indexOf(42n, "1"), -1, "string [0]");
  assert.sameValue(sample.indexOf(43n, "1"), 1, "string [1]");

  assert.sameValue(sample.indexOf(42n, true), -1, "true [0]");
  assert.sameValue(sample.indexOf(43n, true), 1, "true [1]");

  assert.sameValue(sample.indexOf(42n, false), 0, "false [0]");
  assert.sameValue(sample.indexOf(43n, false), 1, "false [1]");

  assert.sameValue(sample.indexOf(42n, NaN), 0, "NaN [0]");
  assert.sameValue(sample.indexOf(43n, NaN), 1, "NaN [1]");

  assert.sameValue(sample.indexOf(42n, null), 0, "null [0]");
  assert.sameValue(sample.indexOf(43n, null), 1, "null [1]");

  assert.sameValue(sample.indexOf(42n, undefined), 0, "undefined [0]");
  assert.sameValue(sample.indexOf(43n, undefined), 1, "undefined [1]");

  assert.sameValue(sample.indexOf(42n, null), 0, "null [0]");
  assert.sameValue(sample.indexOf(43n, null), 1, "null [1]");

  assert.sameValue(sample.indexOf(42n, obj), -1, "object [0]");
  assert.sameValue(sample.indexOf(43n, obj), 1, "object [1]");
});
