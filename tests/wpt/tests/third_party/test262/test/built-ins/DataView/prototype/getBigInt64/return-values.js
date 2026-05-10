// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Return values from Buffer
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  14. Let bufferIndex be getIndex + viewOffset.
  15. Return GetValueFromBuffer(buffer, bufferIndex, type, isLittleEndian).
  ...

  24.1.1.6 GetValueFromBuffer ( arrayBuffer, byteIndex, type [ , isLittleEndian
  ] )

  ...
  8. If isLittleEndian is false, reverse the order of the elements of rawValue.
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

assert.sameValue(sample.getBigInt64(0, false), 0x2702060280008001n, "0, false");
assert.sameValue(sample.getBigInt64(1, false), 0x20602800080017fn, "1, false");
assert.sameValue(sample.getBigInt64(2, false), 0x602800080017f00n, "2, false");
assert.sameValue(sample.getBigInt64(3, false), 0x2800080017f0001n, "3, false");
assert.sameValue(sample.getBigInt64(4, false), -0x7fff7ffe80fffefen, "4, false");
assert.sameValue(sample.getBigInt64(5, false), 0x80017f00010280n, "5, false");
assert.sameValue(sample.getBigInt64(6, false), -0x7ffe80fffefd7f81n, "6, false");
assert.sameValue(sample.getBigInt64(7, false), 0x17f000102807fffn, "7, false");
assert.sameValue(sample.getBigInt64(8, false), 0x7f000102807fff80n, "8, false");

assert.sameValue(sample.getBigInt64(0, true), 0x180008002060227n, "0, true");
assert.sameValue(sample.getBigInt64(1, true), 0x7f01800080020602n, "1, true");
assert.sameValue(sample.getBigInt64(2, true), 0x7f018000800206n, "2, true");
assert.sameValue(sample.getBigInt64(3, true), 0x1007f0180008002n, "3, true");
assert.sameValue(sample.getBigInt64(4, true), 0x201007f01800080n, "4, true");
assert.sameValue(sample.getBigInt64(5, true), -0x7ffdfeff80fe8000n, "5, true");
assert.sameValue(sample.getBigInt64(6, true), 0x7f800201007f0180n, "6, true");
assert.sameValue(sample.getBigInt64(7, true), -0x807ffdfeff80ffn, "7, true");
assert.sameValue(sample.getBigInt64(8, true), -0x7f00807ffdfeff81n, "8, true");
