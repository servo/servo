// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  Throws a RangeError if getIndex < 0
info: |
  24.2.4.14 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView.prototype.getFloat64]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setFloat64(-1, 39);
}, "-1");
assert.sameValue(sample.getFloat64(0), 0, "-1 - no value was set");

assert.throws(RangeError, function() {
  sample.setFloat64(-Infinity, 39);
}, "-Infinity");
assert.sameValue(sample.getFloat64(0), 0, "-Infinity - no value was set");
