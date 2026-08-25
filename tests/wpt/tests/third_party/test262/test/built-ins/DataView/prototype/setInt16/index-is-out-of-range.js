// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint16
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.16 DataView.prototype.setInt16 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Int16", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  11. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  12. Let viewSize be the value of view's [[ByteLength]] internal slot.
  13. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  14. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
features: [DataView.prototype.getInt16]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setInt16(Infinity, 39);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setInt16(13, 39);
}, "13 + 2 > 12");

assert.throws(RangeError, function() {
  sample.setInt16(12, 39);
}, "12 + 2 > 12");

assert.throws(RangeError, function() {
  sample.setInt16(11, 39);
}, "11 + 2 > 12");

sample = new DataView(buffer, 10);
assert.throws(RangeError, function() {
  sample.setInt16(1, 39);
}, "1 + 2 > 2 (offset)");

sample = new DataView(buffer, 11);
assert.throws(RangeError, function() {
  sample.setInt16(0, 39);
}, "0 + 2 > 1 (offset)");

sample = new DataView(buffer, 0, 2);
assert.throws(RangeError, function() {
  sample.setInt16(1, 39);
}, "1 + 2 > 2 (length)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.setInt16(0, 39);
}, "0 + 2 > 1 (length)");

sample = new DataView(buffer, 4, 2);
assert.throws(RangeError, function() {
  sample.setInt16(1, 39);
}, "1 + 2 > 2 (offset+length)");

sample = new DataView(buffer, 4, 1);
assert.throws(RangeError, function() {
  sample.setInt16(0, 39);
}, "0 + 2 > 1 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getInt16(0), 0, "[0] no value was set");
assert.sameValue(sample.getInt16(2), 0, "[1] no value was set");
assert.sameValue(sample.getInt16(4), 0, "[2] no value was set");
assert.sameValue(sample.getInt16(6), 0, "[3] no value was set");
assert.sameValue(sample.getInt16(8), 0, "[4] no value was set");
assert.sameValue(sample.getInt16(10), 0, "[5] no value was set");
