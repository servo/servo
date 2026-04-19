// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.join
description: >
  Concatenates the result of toString for each value with custom separator
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
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1n, 0n, 2n, 3n, 42n, 127n]));

  var result;

  result = sample.join(",");
  assert.sameValue(result, "1,0,2,3,42,127");

  result = sample.join(undefined);
  assert.sameValue(result, "1,0,2,3,42,127");

  result = sample.join(null);
  assert.sameValue(result, "1null0null2null3null42null127");

  result = sample.join(",,");
  assert.sameValue(result, "1,,0,,2,,3,,42,,127");

  result = sample.join(0);
  assert.sameValue(result, "10002030420127");

  result = sample.join("");
  assert.sameValue(result, "102342127");

  result = sample.join(" a b c ");
  assert.sameValue(result, "1 a b c 0 a b c 2 a b c 3 a b c 42 a b c 127");

  result = sample.join({});
  assert.sameValue(result, "1[object Object]0[object Object]2[object Object]3[object Object]42[object Object]127");

  result = sample.join(true);
  assert.sameValue(result, "1true0true2true3true42true127");

  result = sample.join({ toString: function() { return "foo"; }});
  assert.sameValue(result, "1foo0foo2foo3foo42foo127");

  result = sample.join({ toString: undefined, valueOf: function() { return "bar"; }});
  assert.sameValue(result, "1bar0bar2bar3bar42bar127");

  result = sample.join(false);
  assert.sameValue(result, "1false0false2false3false42false127");

  result = sample.join(-1);
  assert.sameValue(result, "1-10-12-13-142-1127");

  result = sample.join(-0);
  assert.sameValue(result, "10002030420127");
});
