// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  10. Let viewOffset be the value of view's [[ByteOffset]] internal slot.
  11. Let viewSize be the value of view's [[ByteLength]] internal slot.
  12. Let elementSize be the Number value of the Element Size value specified in
  Table 50 for Element Type type.
  13. If getIndex + elementSize > viewSize, throw a RangeError exception.
  ...
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, () => sample.getBigInt64(Infinity),
  "DataView access at index Infinity should throw");

assert.throws(RangeError, () => sample.getBigInt64(13), "13 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(12), "12 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(11), "11 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(10), "10 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(9), "9 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(8), "8 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(7), "7 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(6), "6 + 8 > 12");

assert.throws(RangeError, () => sample.getBigInt64(5), "5 + 8 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, () => sample.getBigInt64(1),
  "1 + 8 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, () => sample.getBigInt64(0),
  "0 + 8 > 3 (offset)");

sample = new DataView(buffer, 0, 8);
assert.throws(RangeError, () => sample.getBigInt64(1),
  "1 + 8 > 8 (length)");

sample = new DataView(buffer, 0, 7);
assert.throws(RangeError, () => sample.getBigInt64(0),
  "0 + 8 > 7 (length)");

sample = new DataView(buffer, 4, 8);
assert.throws(RangeError, () => sample.getBigInt64(1),
  "1 + 8 > 8 (offset+length)");

sample = new DataView(buffer, 4, 7);
assert.throws(RangeError, () => sample.getBigInt64(0),
  "0 + 8 > 7 (offset+length)");
