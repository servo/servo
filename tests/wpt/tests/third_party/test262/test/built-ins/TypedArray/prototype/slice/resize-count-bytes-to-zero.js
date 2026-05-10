// Copyright 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.slice
description: >
  Count bytes is set to zero when underlying buffer is resized to zero.
info: |
  %TypedArray%.prototype.slice ( start, end )

  ...
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let srcArrayLength be TypedArrayLength(taRecord).
  4. Let relativeStart be ? ToIntegerOrInfinity(start).
  ...
  14. If countBytes > 0, then
    a. Set taRecord to MakeTypedArrayWithBufferWitnessRecord(O, seq-cst).
    b. If IsTypedArrayOutOfBounds(taRecord) is true, throw a TypeError exception.
    c. Set endIndex to min(endIndex, TypedArrayLength(taRecord)).
    d. Set countBytes to max(endIndex - startIndex, 0).
    ...
includes: [compareArray.js, testTypedArray.js]
features: [resizable-arraybuffer]
---*/

var N = 4;

for (var TA of typedArrayConstructors) {
  var byteLength = N * TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(byteLength, {maxByteLength: byteLength});
  var ta = new TA(buffer);

  for (var startIndex = 0; startIndex <= N; ++startIndex) {
    // Reset typed array and buffer.
    buffer.resize(byteLength);
    ta.fill(1);

    // |start| 
    var start = {
      valueOf() {
        ta.buffer.resize(0);
        return startIndex;
      }
    };

    var sliced = ta.slice(start);

    // Result typed array has the correct length with all elements set to zero.
    assert.compareArray(
      sliced,
      new TA(N - startIndex),
      `${TA.name} with startIndex = ${startIndex}`
    );
  }
}

// Repeat test with different target typed array constructors.
for (var SourceTA of typedArrayConstructors) {
  for (var TargetTA of typedArrayConstructors) {
    var byteLength = N * SourceTA.BYTES_PER_ELEMENT;
    var buffer = new ArrayBuffer(byteLength, {maxByteLength: byteLength});
    var ta = new SourceTA(buffer);

    // Create own "constructor" property to create a different result typed array.
    Object.defineProperty(ta, "constructor", {
      value: TargetTA,
    });

    for (var startIndex = 0; startIndex <= N; ++startIndex) {
      // Reset typed array and buffer.
      buffer.resize(byteLength);
      ta.fill(1);

      var start = {
        valueOf() {
          ta.buffer.resize(0);
          return startIndex;
        }
      };

      var sliced = ta.slice(start);

      // |sliced| is an instance of |TargetTA|.
      assert(
        sliced instanceof TargetTA,
        `is an instance of ${TargetTA.name}`
      );

      // Result typed array has the correct length with all elements set to zero.
      assert.compareArray(
        sliced,
        new TargetTA(N - startIndex),
        `${SourceTA.name} to ${TargetTA.name} with startIndex = ${startIndex}`
      );
    }
  }
}
