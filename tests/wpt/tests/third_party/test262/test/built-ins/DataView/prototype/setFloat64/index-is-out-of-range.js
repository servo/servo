// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat64
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.14 DataView.prototype.setFloat64 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float64", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  11. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  12. Let viewSize be the value of view's [[ByteLength]] internal slot.
  13. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  14. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
features: [DataView.prototype.getFloat64]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setFloat64(Infinity, 39);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setFloat64(13, 39);
}, "13 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(12, 39);
}, "12 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(11, 39);
}, "11 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(10, 39);
}, "10 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(9, 39);
}, "9 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(8, 39);
}, "8 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(7, 39);
}, "7 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(6, 39);
}, "6 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setFloat64(5, 39);
}, "5 + 8 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, function() {
  sample.setFloat64(1, 39);
}, "1 + 8 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, function() {
  sample.setFloat64(0, 39);
}, "0 + 8 > 3 (offset)");

sample = new DataView(buffer, 0, 8);
assert.throws(RangeError, function() {
  sample.setFloat64(1, 39);
}, "1 + 8 > 8 (length)");

sample = new DataView(buffer, 0, 7);
assert.throws(RangeError, function() {
  sample.setFloat64(0, 39);
}, "0 + 8 > 7 (length)");

sample = new DataView(buffer, 4, 8);
assert.throws(RangeError, function() {
  sample.setFloat64(1, 39);
}, "1 + 8 > 8 (offset+length)");

sample = new DataView(buffer, 4, 7);
assert.throws(RangeError, function() {
  sample.setFloat64(0, 39);
}, "0 + 8 > 7 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getFloat64(0), 0, "[0] no value was set");
assert.sameValue(sample.getFloat64(4), 0, "[1] no value was set");
