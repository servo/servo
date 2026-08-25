// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: Return -1 if fromIndex >= ArrayLength - converted values
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements the
  same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.15 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  4. If argument fromIndex was passed, let n be ? ToInteger(fromIndex); else let
  n be len-1.
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
  assert.sameValue(sample.lastIndexOf(42, "1"), 0, "string [0]");
  assert.sameValue(sample.lastIndexOf(43, "1"), 1, "string [1]");

  assert.sameValue(sample.lastIndexOf(42, true), 0, "true [0]");
  assert.sameValue(sample.lastIndexOf(43, true), 1, "true [1]");

  assert.sameValue(sample.lastIndexOf(42, false), 0, "false [0]");
  assert.sameValue(sample.lastIndexOf(43, false), -1, "false [1]");

  assert.sameValue(sample.lastIndexOf(42, NaN), 0, "NaN [0]");
  assert.sameValue(sample.lastIndexOf(43, NaN), -1, "NaN [1]");

  assert.sameValue(sample.lastIndexOf(42, null), 0, "null [0]");
  assert.sameValue(sample.lastIndexOf(43, null), -1, "null [1]");

  assert.sameValue(sample.lastIndexOf(42, undefined), 0, "undefined [0]");
  assert.sameValue(sample.lastIndexOf(43, undefined), -1, "undefined [1]");

  assert.sameValue(sample.lastIndexOf(42, null), 0, "null [0]");
  assert.sameValue(sample.lastIndexOf(43, null), -1, "null [1]");

  assert.sameValue(sample.lastIndexOf(42, obj), 0, "object [0]");
  assert.sameValue(sample.lastIndexOf(43, obj), 1, "object [1]");
});
