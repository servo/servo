// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat64
description: >
  Return -0
info: |
  24.2.4.6 DataView.prototype.getFloat64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float64").

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

sample.setUint8(0, 128);
sample.setUint8(1, 0);
sample.setUint8(2, 0);
sample.setUint8(3, 0);
sample.setUint8(4, 0);
sample.setUint8(5, 0);
sample.setUint8(6, 0);
sample.setUint8(7, 0);

var result = sample.getFloat64(0);
assert.sameValue(result, -0);
