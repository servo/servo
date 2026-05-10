// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  Throws a RangeError if getIndex < 0
info: |
  24.2.4.20 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getUint32]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setUint32(-1, 39);
}, "-1");
assert.sameValue(sample.getUint32(0), 0, "-1 - no value was set");

assert.throws(RangeError, function() {
  sample.setUint32(-Infinity, 39);
}, "-Infinity");
assert.sameValue(sample.getUint32(0), 0, "-Infinity - no value was set");
