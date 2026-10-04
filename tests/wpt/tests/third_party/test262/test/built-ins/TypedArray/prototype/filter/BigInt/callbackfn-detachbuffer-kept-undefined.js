// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Keeping undefined after the callback detaches the buffer throws a TypeError,
  after all callbacks have run.
includes: [detachArrayBuffer.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1n, 2n, 3n]));
  var calls = 0;

  assert.throws(TypeError, function() {
    sample.filter(function(value, index, array) {
      assert.sameValue(index, calls);
      assert.sameValue(array, sample);
      if (calls === 0) {
        assert.sameValue(value, 1n);
        $DETACHBUFFER(sample.buffer);
      } else {
        assert.sameValue(value, undefined,
          "callback element is undefined for index " + index);
      }
      calls++;
      return true;
    });
  });

  assert.sameValue(calls, 3, "All callbacks run before converting the kept values");
}, null, null, ["immutable"]);
