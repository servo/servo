// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: Returns 0 when TA.buffer is immutable
info: |
  Atomics.notify ( typedArray, index, count )
  1. Let taRecord be ? ValidateIntegerTypedArray(typedArray, true).
  2. Let byteIndexInBuffer be ? ValidateAtomicAccess(taRecord, index).
  3. If count is undefined, then
     a. Let c be +∞.
  4. Else,
     a. Let intCount be ? ToIntegerOrInfinity(count).
     b. Let c be max(intCount, 0).
  5. Let buffer be typedArray.[[ViewedArrayBuffer]].
  6. Let block be buffer.[[ArrayBufferData]].
  7. If IsSharedArrayBuffer(buffer) is false, return +0𝔽.
features: [ArrayBuffer, Atomics, immutable-arraybuffer]
includes: [compareArray.js, testTypedArray.js]
---*/

var waitableTypedArrayConstructors = [Int32Array];
if (typeof BigInt64Array !== "undefined") {
  waitableTypedArrayConstructors.push(BigInt64Array);
}

testWithAllTypedArrayConstructors(function(TA, makeCtorArg) {
  var calls = [];
  var index = {
    valueOf() {
      calls.push("index.valueOf");
      return 0;
    }
  };
  var count = {
    valueOf() {
      calls.push("count.valueOf");
      return 1;
    }
  };

  var ta = new TA(makeCtorArg(8));
  assert.sameValue(Atomics.notify(ta, index, count), 0);
  assert.compareArray(calls, ["index.valueOf", "count.valueOf"]);
}, waitableTypedArrayConstructors, ["immutable"]);
