// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Return values from Buffer using a custom offset
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
features: [DataView, ArrayBuffer, DataView.prototype.setUint8, BigInt]
---*/

var buffer = new ArrayBuffer(16);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 0x27);
sample.setUint8(1, 0x02);
sample.setUint8(2, 0x06);
sample.setUint8(3, 0x02);
sample.setUint8(4, 0x80);
sample.setUint8(5, 0x00);
sample.setUint8(6, 0x80);
sample.setUint8(7, 0x01);
sample.setUint8(8, 0x7f);
sample.setUint8(9, 0x00);
sample.setUint8(10, 0x01);
sample.setUint8(11, 0x02);
sample.setUint8(12, 0x80);
sample.setUint8(13, 0x7f);
sample.setUint8(14, 0xff);
sample.setUint8(15, 0x80);

sample = new DataView(buffer, 4);

assert.sameValue(sample.getBigInt64(0, false), -0x7fff7ffe80fffefen, "0, false");
assert.sameValue(sample.getBigInt64(1, false), 0x80017f00010280n, "1, false");
assert.sameValue(sample.getBigInt64(2, false), -0x7ffe80fffefd7f81n, "2, false");
assert.sameValue(sample.getBigInt64(3, false), 0x17f000102807fffn, "3, false");
assert.sameValue(sample.getBigInt64(4, false), 0x7f000102807fff80n, "4, false");

assert.sameValue(sample.getBigInt64(0, true), 0x201007f01800080n, "0, true");
assert.sameValue(sample.getBigInt64(1, true), -0x7ffdfeff80fe8000n, "1, true");
assert.sameValue(sample.getBigInt64(2, true), 0x7f800201007f0180n, "2, true");
assert.sameValue(sample.getBigInt64(3, true), -0x807ffdfeff80ffn, "3, true");
assert.sameValue(sample.getBigInt64(4, true), -0x7f00807ffdfeff81n, "4, true");
