// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.join
description: Concatenates the result of toString for each value
info: |
  22.2.3.15 %TypedArray%.prototype.join ( separator )

  %TypedArray%.prototype.join is a distinct function that implements the same
  algorithm as Array.prototype.join as defined in 22.1.3.13 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length".

  22.1.3.13 Array.prototype.join (separator)

  ...
  7. If element0 is undefined or null, let R be the empty String; otherwise, let
  R be ? ToString(element0).
  8. Let k be 1.
  9. Repeat, while k < len
    a. Let S be the String value produced by concatenating R and sep.
    b. Let element be ? Get(O, ! ToString(k)).
    c. If element is undefined or null, let next be the empty String; otherwise,
    let next be ? ToString(element).
    d. Let R be a String value produced by concatenating S and next.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var arr = [-2, Infinity, NaN, -Infinity, 0.6, 9007199254740992];

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(arr);

  // Use converted values using Array methods as helpers
  var expected = arr.map(function(_, i) {
    return sample[i].toString();
  }).join();

  var result = sample.join();

  assert.sameValue(result, expected);
}, null, ["passthrough"]);
