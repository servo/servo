// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint8
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.15 DataView.prototype.setInt8 ( byteOffset, value )

  1. Let v be the this value.
  2. Return ? SetViewValue(v, byteOffset, true, "Int8", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  11. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  12. Let viewSize be the value of view's [[ByteLength]] internal slot.
  13. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  14. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
features: [DataView.prototype.getInt8]
---*/

var sample;
var buffer = new ArrayBuffer(4);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setInt8(Infinity, 39);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setInt8(5, 39);
}, "5 + 1 > 4");

assert.throws(RangeError, function() {
  sample.setInt8(4, 39);
}, "4 + 1 > 4");

sample = new DataView(buffer, 3);
assert.throws(RangeError, function() {
  sample.setInt8(1, 39);
}, "1 + 1 > 1 (offset)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.setInt8(1, 39);
}, "1 + 1 > 1 (length)");

sample = new DataView(buffer, 2, 1);
assert.throws(RangeError, function() {
  sample.setInt8(1, 39);
}, "1 + 1 > 1 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getInt8(0), 0, "[0] no value was set");
assert.sameValue(sample.getInt8(1), 0, "[1] no value was set");
assert.sameValue(sample.getInt8(2), 0, "[2] no value was set");
assert.sameValue(sample.getInt8(3), 0, "[3] no value was set");
