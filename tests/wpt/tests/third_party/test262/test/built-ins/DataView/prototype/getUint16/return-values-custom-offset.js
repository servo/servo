// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint16
description: >
  Return values from Buffer using a custom offset
info: |
  24.2.4.11 DataView.prototype.getUint16 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint16").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  14. Let bufferIndex be getIndex + viewOffset.
  15. Return GetValueFromBuffer(buffer, bufferIndex, type, isLittleEndian).
  ...

  24.1.1.5 GetValueFromBuffer ( arrayBuffer, byteIndex, type [ , isLittleEndian
  ] )

  ...
  8. If isLittleEndian is false, reverse the order of the elements of rawValue.
  ...
features: [DataView.prototype.setUint8]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

sample.setUint8(0, 0);
sample.setUint8(1, 2);
sample.setUint8(2, 6);
sample.setUint8(3, 2);
sample.setUint8(4, 128);
sample.setUint8(5, 42);
sample.setUint8(6, 128);
sample.setUint8(7, 39);

sample = new DataView(buffer, 4);

assert.sameValue(sample.getUint16(0, false), 32810, "0, false");
assert.sameValue(sample.getUint16(1, false), 10880, "1, false");
assert.sameValue(sample.getUint16(2, false), 32807, "2, false");

assert.sameValue(sample.getUint16(0, true), 10880, "0, true");
assert.sameValue(sample.getUint16(1, true), 32810, "1, true");
assert.sameValue(sample.getUint16(2, true), 10112, "2, true");
