// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Throws a RangeError if getIndex + elementSize > viewSize
features: [DataView, ArrayBuffer, BigInt]
---*/

var sample;
var buffer = new ArrayBuffer(12);

sample = new DataView(buffer, 0);

assert.throws(RangeError, function() {
  sample.setBigInt64(Infinity, 39n);
}, "getIndex == Infinity");

assert.throws(RangeError, function() {
  sample.setBigInt64(13, 39n);
}, "13 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(12, 39n);
}, "12 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(11, 39n);
}, "11 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(10, 39n);
}, "10 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(9, 39n);
}, "9 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(8, 39n);
}, "8 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(7, 39n);
}, "7 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(6, 39n);
}, "6 + 8 > 12");

assert.throws(RangeError, function() {
  sample.setBigInt64(5, 39n);
}, "5 + 8 > 12");

sample = new DataView(buffer, 8);
assert.throws(RangeError, function() {
  sample.setBigInt64(1, 39n);
}, "1 + 8 > 4 (offset)");

sample = new DataView(buffer, 9);
assert.throws(RangeError, function() {
  sample.setBigInt64(0, 39n);
}, "0 + 8 > 3 (offset)");

sample = new DataView(buffer, 0, 8);
assert.throws(RangeError, function() {
  sample.setBigInt64(1, 39n);
}, "1 + 8 > 8 (length)");

sample = new DataView(buffer, 0, 7);
assert.throws(RangeError, function() {
  sample.setBigInt64(0, 39n);
}, "0 + 8 > 7 (length)");

sample = new DataView(buffer, 4, 8);
assert.throws(RangeError, function() {
  sample.setBigInt64(1, 39n);
}, "1 + 8 > 8 (offset+length)");

sample = new DataView(buffer, 4, 7);
assert.throws(RangeError, function() {
  sample.setBigInt64(0, 39n);
}, "0 + 8 > 7 (offset+length)");

sample = new DataView(buffer, 0);
assert.sameValue(sample.getBigInt64(0), 0n, "[0] no value was set");
assert.sameValue(sample.getBigInt64(4), 0n, "[1] no value was set");
