// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
features: [Float16Array]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.getFloat16(Infinity);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.getFloat16(13);
}, "13 + 2 > 12");

assert.throws(RangeError, function() {
  sample.getFloat16(12);
}, "12 + 2 > 12");

assert.throws(RangeError, function() {
  sample.getFloat16(11);
}, "11 + 2 > 12");

sample = new DataView(buffer, 10);
assert.throws(RangeError, function() {
  sample.getFloat16(1);
}, "1 + 2 > 2 (offset)");

sample = new DataView(buffer, 11);
assert.throws(RangeError, function() {
  sample.getFloat16(0);
}, "0 + 2 > 1 (offset)");

sample = new DataView(buffer, 0, 2);
assert.throws(RangeError, function() {
  sample.getFloat16(1);
}, "1 + 2 > 2 (length)");

sample = new DataView(buffer, 0, 1);
assert.throws(RangeError, function() {
  sample.getFloat16(0);
}, "0 + 2 > 1 (length)");

sample = new DataView(buffer, 4, 2);
assert.throws(RangeError, function() {
  sample.getFloat16(1);
}, "1 + 2 > 2 (offset+length)");

sample = new DataView(buffer, 4, 1);
assert.throws(RangeError, function() {
  sample.getFloat16(0);
}, "0 + 2 > 1 (offset+length)");
