// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint8
description: >
  Throws a RangeError if getIndex < 0
info: |
  24.2.4.18 DataView.prototype.setUint8 ( byteOffset, value )

  1. Let v be the this value.
  2. Return ? SetViewValue(v, byteOffset, true, "Uint8", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [Uint8Array]
---*/

var buffer = new ArrayBuffer(2);
var sample = new DataView(buffer, 0);
var typedArray = new Uint8Array(buffer, 0);

assert.throws(RangeError, function() {
  sample.setUint8(-1, 39);
}, "-1");
assert.sameValue(typedArray[0], 0, "-1 - no value was set");

assert.throws(RangeError, function() {
  sample.setUint8(-Infinity, 39);
}, "-Infinity");
assert.sameValue(typedArray[0], 0, "-Infinity - no value was set");
