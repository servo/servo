// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
features: [Float16Array]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setFloat16(Infinity, 39);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setFloat16(13, 39);
}, "13 + 2 > 12");

assert.throws(RangeError, function() {
  sample.setFloat16(12, 39);
}, "12 + 2 > 12");

assert.throws(RangeError, function() {
  sample.setFloat16(11, 39);
}, "11 + 2 > 12");

sample = new DataView(buffer, 10);
assert.throws(RangeError, function() {
  sample.setFloat16(1, 39);
}, "1 + 2 > 2 (offset)");

sample = new DataView(buffer, 11);
assert.throws(RangeError, function() {
  sample.setFloat16(0, 39);
}, "0 + 2 > 1 (offset)");

sample = new DataView(buffer, 0, 2);
assert.throws(RangeError, function() {
  sample.setFloat16(1, 39);
}, "1 + 2 > 2 (length)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.setFloat16(0, 39);
}, "0 + 2 > 1 (length)");

sample = new DataView(buffer, 4, 2);
assert.throws(RangeError, function() {
  sample.setFloat16(1, 39);
}, "1 + 2 > 2 (offset+length)");

sample = new DataView(buffer, 4, 1);
assert.throws(RangeError, function() {
  sample.setFloat16(0, 39);
}, "0 + 2 > 1 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getFloat16(0), 0, "[0] no value was set");
assert.sameValue(sample.getFloat16(2), 0, "[1] no value was set");
assert.sameValue(sample.getFloat16(4), 0, "[2] no value was set");
assert.sameValue(sample.getFloat16(6), 0, "[3] no value was set");
assert.sameValue(sample.getFloat16(8), 0, "[4] no value was set");
assert.sameValue(sample.getFloat16(10), 0, "[5] no value was set");
