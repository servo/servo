/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Test DataView on SharedArrayBuffer.

if (this.SharedArrayBuffer) {

var sab = new SharedArrayBuffer(4096);
var dv = new DataView(sab);

assert.sameValue(sab, dv.buffer);
assert.sameValue(dv.byteLength, sab.byteLength);
assert.sameValue(ArrayBuffer.isView(dv), true);

var dv2 = new DataView(sab, 1075, 2048);

assert.sameValue(sab, dv2.buffer);
assert.sameValue(dv2.byteLength, 2048);
assert.sameValue(dv2.byteOffset, 1075);
assert.sameValue(ArrayBuffer.isView(dv2), true);

// Test that it is the same buffer memory for the two views

dv.setInt8(1075, 37);
assert.sameValue(dv2.getInt8(0), 37);

// Test that range checking works

assert.throws(RangeError, () => dv.setInt32(4098, -1));
assert.throws(RangeError, () => dv.setInt32(4094, -1));
assert.throws(RangeError, () => dv.setInt32(-1, -1));

assert.throws(RangeError, () => dv2.setInt32(2080, -1));
assert.throws(RangeError, () => dv2.setInt32(2046, -1));
assert.throws(RangeError, () => dv2.setInt32(-1, -1));

}

