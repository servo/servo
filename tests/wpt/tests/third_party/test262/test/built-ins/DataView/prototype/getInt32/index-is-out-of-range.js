// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint32
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.9 DataView.prototype.getInt32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  10. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  11. Let viewSize be the value of view's [[ByteLength]] internal slot.
  12. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  13. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.getInt32(Infinity);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.getInt32(13);
}, "13 + 4 > 12");

assert.throws(RangeError, function() {
  sample.getInt32(12);
}, "12 + 4 > 12");

assert.throws(RangeError, function() {
  sample.getInt32(11);
}, "11 + 4 > 12");

assert.throws(RangeError, function() {
  sample.getInt32(10);
}, "10 + 4 > 12");

assert.throws(RangeError, function() {
  sample.getInt32(9);
}, "9 + 4 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, function() {
  sample.getInt32(1);
}, "1 + 4 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, function() {
  sample.getInt32(0);
}, "0 + 4 > 3 (offset)");

sample = new DataView(buffer, 0, 4);
assert.throws(RangeError, function() {
  sample.getInt32(1);
}, "1 + 4 > 4 (length)");

sample = new DataView(buffer, 0, 3);
assert.throws(RangeError, function() {
  sample.getInt32(0);
}, "0 + 4 > 3 (length)");

sample = new DataView(buffer, 4, 4);
assert.throws(RangeError, function() {
  sample.getInt32(1);
}, "1 + 4 > 4 (offset+length)");

sample = new DataView(buffer, 4, 3);
assert.throws(RangeError, function() {
  sample.getInt32(0);
}, "0 + 4 > 3 (offset+length)");
