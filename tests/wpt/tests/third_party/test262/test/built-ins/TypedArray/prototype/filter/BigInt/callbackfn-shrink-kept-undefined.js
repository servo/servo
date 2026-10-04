// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Keeping undefined after the callback shrinks the buffer throws a TypeError,
  after all callbacks have run.
includes: [testTypedArray.js]
features: [BigInt, TypedArray, resizable-arraybuffer]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  [false, true].forEach(function(fixedLength) {
    var buffer = new ArrayBuffer(3 * TA.BYTES_PER_ELEMENT, {
      maxByteLength: 3 * TA.BYTES_PER_ELEMENT
    });
    var sample = fixedLength ? new TA(buffer, 0, 3) : new TA(buffer);
    sample[0] = 1n;
    sample[1] = 2n;
    sample[2] = 3n;
    var calls = 0;

    assert.throws(TypeError, function() {
      sample.filter(function(value, index, array) {
        assert.sameValue(index, calls);
        assert.sameValue(array, sample);
        if (calls === 0) {
          assert.sameValue(value, 1n);
          buffer.resize(TA.BYTES_PER_ELEMENT);
        } else {
          assert.sameValue(value, undefined,
            "callback element is undefined for index " + index);
        }
        calls++;
        return true;
      });
    });

    assert.sameValue(calls, 3, "All callbacks run before converting the kept values");
  });
}, null, ["passthrough"]);
