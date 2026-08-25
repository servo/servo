// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setuint32
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.20 DataView.prototype.setUint32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Uint32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  ...
  11. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  12. Let viewSize be the value of view's [[ByteLength]] internal slot.
  13. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  14. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
features: [DataView.prototype.getUint32]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setUint32(Infinity, 39);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setUint32(13, 39);
}, "13 + 4 > 12");

assert.throws(RangeError, function() {
  sample.setUint32(12, 39);
}, "12 + 4 > 12");

assert.throws(RangeError, function() {
  sample.setUint32(11, 39);
}, "11 + 4 > 12");

assert.throws(RangeError, function() {
  sample.setUint32(10, 39);
}, "10 + 4 > 12");

assert.throws(RangeError, function() {
  sample.setUint32(9, 39);
}, "9 + 4 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, function() {
  sample.setUint32(1, 39);
}, "1 + 4 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, function() {
  sample.setUint32(0, 39);
}, "0 + 4 > 3 (offset)");

sample = new DataView(buffer, 0, 4);
assert.throws(RangeError, function() {
  sample.setUint32(1, 39);
}, "1 + 4 > 4 (length)");

sample = new DataView(buffer, 0, 3);
assert.throws(RangeError, function() {
  sample.setUint32(0, 39);
}, "0 + 4 > 3 (length)");

sample = new DataView(buffer, 4, 4);
assert.throws(RangeError, function() {
  sample.setUint32(1, 39);
}, "1 + 4 > 4 (offset+length)");

sample = new DataView(buffer, 4, 3);
assert.throws(RangeError, function() {
  sample.setUint32(0, 39);
}, "0 + 4 > 3 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getUint32(0), 0, "[0] no value was set");
assert.sameValue(sample.getUint32(4), 0, "[1] no value was set");
assert.sameValue(sample.getUint32(8), 0, "[2] no value was set");
