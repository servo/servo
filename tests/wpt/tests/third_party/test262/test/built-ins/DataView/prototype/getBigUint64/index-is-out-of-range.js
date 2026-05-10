// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, () => sample.getBigUint64(Infinity),
  "DataView access at index Infinity should throw");

assert.throws(RangeError, () => sample.getBigUint64(13), "13 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(12), "12 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(11), "11 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(10), "10 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(9), "9 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(8), "8 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(7), "7 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(6), "6 + 8 > 12");

assert.throws(RangeError, () => sample.getBigUint64(5), "5 + 8 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, () => sample.getBigUint64(1),
  "1 + 8 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, () => sample.getBigUint64(0),
  "0 + 8 > 3 (offset)");

sample = new DataView(buffer, 0, 8);
assert.throws(RangeError, () => sample.getBigUint64(1),
  "1 + 8 > 8 (length)");

sample = new DataView(buffer, 0, 7);
assert.throws(RangeError, () => sample.getBigUint64(0),
  "0 + 8 > 7 (length)");

sample = new DataView(buffer, 4, 8);
assert.throws(RangeError, () => sample.getBigUint64(1),
  "1 + 8 > 8 (offset+length)");

sample = new DataView(buffer, 4, 7);
assert.throws(RangeError, () => sample.getBigUint64(0),
  "0 + 8 > 7 (offset+length)");
