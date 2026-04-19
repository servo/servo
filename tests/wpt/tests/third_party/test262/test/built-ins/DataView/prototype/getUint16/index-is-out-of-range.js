// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint16
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.11 DataView.prototype.getUint16 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Uint16").

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
  sample.getUint16(Infinity);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.getUint16(13);
}, "13 + 2 > 12");

assert.throws(RangeError, function() {
  sample.getUint16(12);
}, "12 + 2 > 12");

assert.throws(RangeError, function() {
  sample.getUint16(11);
}, "11 + 2 > 12");

sample = new DataView(buffer, 10);
assert.throws(RangeError, function() {
  sample.getUint16(1);
}, "1 + 2 > 2 (offset)");

sample = new DataView(buffer, 11);
assert.throws(RangeError, function() {
  sample.getUint16(0);
}, "0 + 2 > 1 (offset)");

sample = new DataView(buffer, 0, 2);
assert.throws(RangeError, function() {
  sample.getUint16(1);
}, "1 + 2 > 2 (length)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.getUint16(0);
}, "0 + 2 > 1 (length)");

sample = new DataView(buffer, 4, 2);
assert.throws(RangeError, function() {
  sample.getUint16(1);
}, "1 + 2 > 2 (offset+length)");

sample = new DataView(buffer, 4, 1);
assert.throws(RangeError, function() {
  sample.getUint16(0);
}, "0 + 2 > 1 (offset+length)");
