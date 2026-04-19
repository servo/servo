// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  Set values on the little endian order
info: |
  24.2.4.20 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  15. Let bufferIndex be getIndex + viewOffset.
  16. Return SetValueInBuffer(buffer, bufferIndex, type, numberValue, isLittleEndian).

  24.1.1.6 SetValueInBuffer ( arrayBuffer, byteIndex, type, value [ , isLittleEndian ] )

  ...
  11. Store the individual bytes of rawBytes into block, in order, starting at
  block[byteIndex].
  12. Return NormalCompletion(undefined).
features: [DataView.prototype.getUint32]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

var result;

result = sample.setUint32(0, -1870724872, true);
assert.sameValue(result, undefined, "returns undefined #1");
assert.sameValue(sample.getUint32(0), 4160782224);

result = sample.setUint32(0, -134185072, true);
assert.sameValue(result, undefined, "returns undefined #2");
assert.sameValue(sample.getUint32(0), 2424242424);

result = sample.setUint32(0, 1870724872, true);
assert.sameValue(result, undefined, "returns undefined #3");
assert.sameValue(sample.getUint32(0), 150962287);

result = sample.setUint32(0, 150962287, true);
assert.sameValue(result, undefined, "returns undefined #4");
assert.sameValue(sample.getUint32(0), 1870724872);

result = sample.setUint32(0, 4160782224, true);
assert.sameValue(result, undefined, "returns undefined #5");
assert.sameValue(sample.getUint32(0), 2424242424);

result = sample.setUint32(0, 2424242424, true);
assert.sameValue(result, undefined, "returns undefined #6");
assert.sameValue(sample.getUint32(0), 4160782224);
