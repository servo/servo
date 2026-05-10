// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  Set value as undefined (cast to 0) when value argument is not present
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

sample.setUint32(0, 42);

var result = sample.setUint32(0);

assert.sameValue(sample.getUint32(0), 0);
assert.sameValue(result, undefined);
