// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat32
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.13 DataView.prototype.setFloat32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ , isLittleEndian ] )

  ...
  8. If type is "Float32", then
    a. Set rawBytes to a List containing the 4 bytes that are the result of
    converting value to IEEE 754-2008 binary32 format using “Round to nearest,
    ties to even” rounding mode. If isLittleEndian is false, the bytes are
    arranged in big endian order. Otherwise, the bytes are arranged in little
    endian order. [...]
  ...
features: [DataView.prototype.getFloat32, Symbol]
---*/

var buffer = new ArrayBuffer(4);
var sample = new DataView(buffer, 0);

// False
sample.setFloat32(0, 1);
assert.sameValue(sample.getFloat32(0), 1, "no arg");
sample.setFloat32(0, 2, undefined);
assert.sameValue(sample.getFloat32(0), 2, "undefined");
sample.setFloat32(0, 3, null);
assert.sameValue(sample.getFloat32(0), 3, "null");
sample.setFloat32(0, 4, 0);
assert.sameValue(sample.getFloat32(0), 4, "0");
sample.setFloat32(0, 5, "");
assert.sameValue(sample.getFloat32(0), 5, "the empty string");

// True
sample.setFloat32(0, 6, {});
assert.sameValue(sample.getFloat32(0), 6.89663052202102e-41, "{}");
sample.setFloat32(0, 7, Symbol("1"));
assert.sameValue(sample.getFloat32(0), 8.04457422399591e-41, "symbol");
sample.setFloat32(0, 8, 1);
assert.sameValue(sample.getFloat32(0), 9.10844001811131e-44, "1");
sample.setFloat32(0, 9, "string");
assert.sameValue(sample.getFloat32(0), 5.830802910055564e-42, "string");
