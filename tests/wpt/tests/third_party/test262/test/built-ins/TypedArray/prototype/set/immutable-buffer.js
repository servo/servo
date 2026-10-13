// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  %TypedArray%.prototype.set ( source [ , offset ] )
  1. Let target be the this value.
  ...
  3. Perform ? RequireInternalSlot(target, [[TypedArrayName]]).
  4. Assert: target has a [[ViewedArrayBuffer]] internal slot.
  5. If IsImmutableBuffer(target.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
  6. Let targetOffset be ? ToIntegerOrInfinity(offset).
features: [TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];

  var ta = new TA(makeCtorArg(["1", "2", "3", "4"]));
  var source = {
    get length() {
      calls.push("get source.length");
      return 1;
    },
    get 0() {
      calls.push("get source[0]");
      return "8";
    },
  };
  var offset = {
    valueOf() {
      calls.push("offset.valueOf");
      return 1;
    }
  };

  assert.throws(TypeError, function() {
    ta.set(source, offset);
  });
  assert.compareArray(calls, [], "Must verify mutability before reading arguments.");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]), "Must not mutate contents.");
}, null, ["immutable"]);
