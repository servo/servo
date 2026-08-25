// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Return value from Buffer using a clean ArrayBuffer
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  12. Let bufferIndex be getIndex + viewOffset.
  13. Return GetValueFromBuffer(buffer, bufferIndex, type, false,
     "Unordered", isLittleEndian).

  24.1.1.6 GetValueFromBuffer ( arrayBuffer, byteIndex, type,
  isTypedArray, order [ , isLittleEndian ] )

  ...
  9. Return RawBytesToNumber(type, rawValue, isLittleEndian).

  24.1.1.5 RawBytesToNumber( type, rawBytes, isLittleEndian )

  ...
  2. If isLittleEndian is false, reverse the order of the elements of rawBytes.
  ...
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.sameValue(sample.getBigInt64(0, true), 0n, "sample.getBigInt64(0, true)");
assert.sameValue(sample.getBigInt64(1, true), 0n, "sample.getBigInt64(1, true)");
assert.sameValue(sample.getBigInt64(2, true), 0n, "sample.getBigInt64(2, true)");
assert.sameValue(sample.getBigInt64(3, true), 0n, "sample.getBigInt64(3, true)");
assert.sameValue(sample.getBigInt64(4, true), 0n, "sample.getBigInt64(4, true)");
assert.sameValue(sample.getBigInt64(0, false), 0n, "sample.getBigInt64(0, false)");
assert.sameValue(sample.getBigInt64(1, false), 0n, "sample.getBigInt64(1, false)");
assert.sameValue(sample.getBigInt64(2, false), 0n, "sample.getBigInt64(2, false)");
assert.sameValue(sample.getBigInt64(3, false), 0n, "sample.getBigInt64(3, false)");
assert.sameValue(sample.getBigInt64(4, false), 0n, "sample.getBigInt64(4, false)");
