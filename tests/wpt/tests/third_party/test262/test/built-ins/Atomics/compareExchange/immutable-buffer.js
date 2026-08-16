// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.compareexchange
description: >
  Throws a TypeError exception when the backing buffer is immutable
info: |
  Atomics.compareExchange ( typedArray, index, expectedValue, replacementValue )
  1. Let byteIndexInBuffer be ? ValidateAtomicAccessOnIntegerTypedArray(typedArray, index, false, ~write~).

  ValidateAtomicAccessOnIntegerTypedArray ( typedArray, requestIndex [ , waitable [ , accessMode ] ] )
  1. If waitable is not present, set waitable to false.
  2. If accessMode is not present, set accessMode to ~read~.
  3. Let taRecord be ? ValidateIntegerTypedArray(typedArray, waitable, accessMode).

  ValidateIntegerTypedArray ( typedArray, waitable [ , accessMode ] )
  1. If accessMode is not present, set accessMode to ~read~.
  2. Let taRecord be ? ValidateTypedArray(typedArray, unordered, accessMode).

  ValidateTypedArray ( O, order [ , accessMode ] )
  ...
  4. If accessMode is ~write~ and IsImmutableBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
features: [Atomics, immutable-arraybuffer]
includes: [compareArray.js, testTypedArray.js]
---*/

testWithAllTypedArrayConstructors(function(TA, makeCtorArg) {
  var calls = [];
  var index = {
    valueOf() {
      calls.push("index.valueOf");
      return 0;
    }
  };
  var expectedValue = {
    valueOf() {
      calls.push("expectedValue.valueOf");
      return 0;
    }
  };
  var replacementValue = {
    valueOf() {
      calls.push("replacementValue.valueOf");
      return 1;
    }
  };

  var ta = new TA(makeCtorArg(8));
  assert.throws(TypeError, function() {
    Atomics.compareExchange(ta, index, expectedValue, replacementValue);
  });
  assert.compareArray(calls, []);
}, null, ["immutable"]);
