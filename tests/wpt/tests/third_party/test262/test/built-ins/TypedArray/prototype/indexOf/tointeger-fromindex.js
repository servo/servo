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
features: [TypedArray]
---*/

var obj = {
  valueOf: function() {
    return 1;
  }
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample;

  sample = new TA(makeCtorArg([42, 43]));
  assert.sameValue(sample.indexOf(42, "1"), -1, "string [0]");
  assert.sameValue(sample.indexOf(43, "1"), 1, "string [1]");

  assert.sameValue(sample.indexOf(42, true), -1, "true [0]");
  assert.sameValue(sample.indexOf(43, true), 1, "true [1]");

  assert.sameValue(sample.indexOf(42, false), 0, "false [0]");
  assert.sameValue(sample.indexOf(43, false), 1, "false [1]");

  assert.sameValue(sample.indexOf(42, NaN), 0, "NaN [0]");
  assert.sameValue(sample.indexOf(43, NaN), 1, "NaN [1]");

  assert.sameValue(sample.indexOf(42, null), 0, "null [0]");
  assert.sameValue(sample.indexOf(43, null), 1, "null [1]");

  assert.sameValue(sample.indexOf(42, undefined), 0, "undefined [0]");
  assert.sameValue(sample.indexOf(43, undefined), 1, "undefined [1]");

  assert.sameValue(sample.indexOf(42, null), 0, "null [0]");
  assert.sameValue(sample.indexOf(43, null), 1, "null [1]");

  assert.sameValue(sample.indexOf(42, obj), -1, "object [0]");
  assert.sameValue(sample.indexOf(43, obj), 1, "object [1]");
});
