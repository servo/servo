// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-integer-indexed-exotic-objects-ownpropertykeys
description: >
  List not-enumerable own keys
info: |
  9.4.5.6 [[OwnPropertyKeys]] ()

  ...
  3. Let len be the value of O's [[ArrayLength]] internal slot.
  4. For each integer i starting with 0 such that i < len, in ascending order,
    a. Add ! ToString(i) as the last element of keys.
  ...
includes: [testTypedArray.js, compareArray.js]
features: [Reflect, Symbol, TypedArray]
---*/

var s = Symbol("1");

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA();

  Object.defineProperty(sample, s, {
    value: 42,
    enumerable: false
  });
  Object.defineProperty(sample, "test262", {
    value: 42,
    enumerable: false
  });
  var result = Reflect.ownKeys(sample);
  assert(compareArray(result, ["test262", s]));
}, null, ["passthrough"]);
