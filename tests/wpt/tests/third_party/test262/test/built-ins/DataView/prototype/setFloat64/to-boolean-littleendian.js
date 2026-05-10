// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  Boolean littleEndian argument coerced in ToBoolean
info: |
  24.2.4.14 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ , isLittleEndian ] )

  ...
  9. Else if type is "Float64", then
    a. Set rawBytes to a List containing the 8 bytes that are the IEEE 754-2008
    binary64 format encoding of value. If isLittleEndian is false, the bytes are
    arranged in big endian order. Otherwise, the bytes are arranged in little
    endian order. [...]
  ...
features: [DataView.prototype.getFloat64, Symbol]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

// False
sample.setFloat64(0, 1);
assert.sameValue(sample.getFloat64(0), 1, "no arg");
sample.setFloat64(0, 2, undefined);
assert.sameValue(sample.getFloat64(0), 2, "undefined");
sample.setFloat64(0, 3, null);
assert.sameValue(sample.getFloat64(0), 3, "null");
sample.setFloat64(0, 4, 0);
assert.sameValue(sample.getFloat64(0), 4, "0");
sample.setFloat64(0, 5, "");
assert.sameValue(sample.getFloat64(0), 5, "the empty string");

// True
sample.setFloat64(0, 3.067e-320, {});
assert.sameValue(sample.getFloat64(0), 6, "{}");
sample.setFloat64(0, 3.573e-320, Symbol("1"));
assert.sameValue(sample.getFloat64(0), 7, "symbol");
sample.setFloat64(0, 4.079e-320, 1);
assert.sameValue(sample.getFloat64(0), 8, "1");
sample.setFloat64(0, 4.332e-320, "string");
assert.sameValue(sample.getFloat64(0), 9, "string");
