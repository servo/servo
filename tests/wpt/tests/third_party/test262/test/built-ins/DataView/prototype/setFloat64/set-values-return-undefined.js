// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  Set values and return undefined
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
    endian order. If value is NaN, rawValue may be set to any implementation
    chosen IEEE 754-2008 binary64 format Not-a-Number encoding. An
    implementation must always choose the same encoding for each implementation
    distinguishable NaN value.
  ...
  11. Store the individual bytes of rawBytes into block, in order, starting at
  block[byteIndex].
  12. Return NormalCompletion(undefined).
includes: [byteConversionValues.js]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

var values = byteConversionValues.values;
var expectedValues = byteConversionValues.expected.Float64;

values.forEach(function(value, i) {
  var expected = expectedValues[i];

  var result = sample.setFloat64(0, value, false);

  assert.sameValue(
    sample.getFloat64(0),
    expected,
    "value: " + value
  );
  assert.sameValue(
    result,
    undefined,
    "return is undefined, value: " + value
  );
});
