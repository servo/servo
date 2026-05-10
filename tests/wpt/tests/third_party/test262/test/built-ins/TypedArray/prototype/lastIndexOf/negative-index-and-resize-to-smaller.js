// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.lastindexof
description: >
  Negative index is relative to the original typed array length.
info: |
  %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  5. If fromIndex is present, let n be ? ToIntegerOrInfinity(fromIndex); else let n be len - 1.
  6. If n = -∞, return -1𝔽.
  7. If n ≥ 0, then
    a. Let k be min(n, len - 1).
  8. Else,
    a. Let k be len + n.
  ...
features: [TypedArray, resizable-arraybuffer]
includes: [testTypedArray.js]
---*/

testWithTypedArrayConstructors(function(TA) {
  var byteLength = 4 * TA.BYTES_PER_ELEMENT;
  var rab = new ArrayBuffer(0, {maxByteLength: byteLength});
  var ta = new TA(rab);

  var indices = [
    [-1, 2],
    [-2, 2],
    [-3, 1],
    [-4, 0],
    [-5, -1],
  ];

  for (var i = 0; i < indices.length; ++i) {
    var index = indices[i][0];
    var expected = indices[i][1];
    var searchElement = 123;

    rab.resize(byteLength);
    ta.fill(searchElement);

    var indexValue = {
      valueOf() {
        rab.resize(3 * TA.BYTES_PER_ELEMENT);
        return index;
      }
    };

    assert.sameValue(
      ta.lastIndexOf(searchElement, indexValue),
      expected,
      "For index " + index
    );
  }
}, null, ["passthrough"]);
