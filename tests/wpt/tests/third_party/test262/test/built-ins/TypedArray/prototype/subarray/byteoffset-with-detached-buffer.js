// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.subarray
description: >
  Species constructor is called with the correct byte-offset value
info: |
  %TypedArray%.prototype.subarray ( start, end )

  ...
  13. Let srcByteOffset be O.[[ByteOffset]].
  14. Let beginByteOffset be srcByteOffset + (startIndex × elementSize).
  ...
  16. Else,
    ...
    f. Let argumentsList be « buffer, 𝔽(beginByteOffset), 𝔽(newLength) ».
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).
features: [TypedArray]
includes: [testTypedArray.js, detachArrayBuffer.js]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ab = new ArrayBuffer(2 * TA.BYTES_PER_ELEMENT);
  var ta = new TA(ab, TA.BYTES_PER_ELEMENT, 1);
  var result = new TA(0);

  ta.constructor = {
    [Symbol.species]: function(buffer, byteOffset, length) {
      assert.sameValue(buffer, ab);
      assert.sameValue(byteOffset, 2 * TA.BYTES_PER_ELEMENT);
      assert.sameValue(length, 0);
      return result;
    }
  };

  var end = {
    valueOf() {
      $DETACHBUFFER(ab);
      return 0;
    }
  };

  assert.sameValue(ta.subarray(1, end), result);
}, null, ["passthrough"]);
