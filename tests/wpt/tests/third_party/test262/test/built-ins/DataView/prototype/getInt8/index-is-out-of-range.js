// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint8
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  24.2.4.7 DataView.prototype.getInt8 ( byteOffset )

  1. Let v be the this value.
  2. Return ? GetViewValue(v, byteOffset, true, "Int8").

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
  sample.getInt8(Infinity);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.getInt8(13);
}, "13 + 1 > 12");

assert.throws(RangeError, function() {
  sample.getInt8(12);
}, "12 + 1 > 12");

sample = new DataView(buffer, 11);
assert.throws(RangeError, function() {
  sample.getInt8(1);
}, "1 + 1 > 1 (offset)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.getInt8(1);
}, "1 + 1 > 1 (length)");

sample = new DataView(buffer, 4, 1);
assert.throws(RangeError, function() {
  sample.getInt8(1);
}, "1 + 1 > 1 (offset+length)");

sample = new DataView(buffer, 4, 0);
assert.throws(RangeError, function() {
  sample.getInt8(0);
}, "0 + 1 > 0 (offset+length)");
