// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-ownpropertykeys
description: >
  Return keys
info: |
  9.4.5.6 [[OwnPropertyKeys]] ()

  ...
  3. Let len be the value of O's [[ArrayLength]] internal slot.
  4. For each integer i starting with 0 such that i < len, in ascending order,
    a. Add ! ToString(i) as the last element of keys.
  ...
includes: [testTypedArray.js, compareArray.js]
features: [Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample1 = new TA(makeCtorArg([42, 42, 42]));
  var result1 = Reflect.ownKeys(sample1);
  assert(compareArray(result1, ["0", "1", "2"]), "result1");

  var sample2 = new TA(makeCtorArg(4));
  var result2 = Reflect.ownKeys(sample2);
  assert(compareArray(result2, ["0", "1", "2", "3"]), "result2");

  var sample3 = new TA(makeCtorArg(4)).subarray(2);
  var result3 = Reflect.ownKeys(sample3);
  assert(compareArray(result3, ["0", "1"]), "result3");

  var sample4 = new TA();
  var result4 = Reflect.ownKeys(sample4);
  assert(compareArray(result4, []), "result4");
});
