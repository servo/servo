// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint16
description: >
  Set values and return undefined
info: |
  24.2.4.16 DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int16", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ , isLittleEndian ] )

  ...
  11. Store the individual bytes of rawBytes into block, in order, starting at
  block[byteIndex].
  12. Return NormalCompletion(undefined).
features: [DataView.prototype.getInt16]
includes: [byteConversionValues.js]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

var values = byteConversionValues.values;
var expectedValues = byteConversionValues.expected.Int16;

values.forEach(function(value, i) {
  var expected = expectedValues[i];

  var result = sample.setInt16(0, value, false);

  assert.sameValue(
    sample.getInt16(0),
    expected,
    "value: " + value
  );
  assert.sameValue(
    result,
    undefined,
    "return is undefined, value: " + value
  );
});
