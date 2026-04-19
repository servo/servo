// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a RangeError if ToInteger(byteLength) < 0
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  8. If byteLength is either not present or undefined, then
    a. Let viewByteLength be bufferByteLength - offset.
  9. Else,
    a. Let viewByteLength be ? ToIndex(byteLength).
  ...

  ToIndex ( value )

  1. If value is undefined, then
    a. Let index be 0.
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    ...
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(2);

assert.throws(RangeError, function() {
  new DataView(buffer, 0, -1);
}, "new DataView(buffer, 0, -1);");

assert.throws(RangeError, function() {
  new DataView(buffer, 0, -Infinity);
}, "new DataView(buffer, 0, -Infinity);");

assert.throws(RangeError, function() {
  new DataView(buffer, 1, -1);
}, "new DataView(buffer, 1, -1);");

assert.throws(RangeError, function() {
  new DataView(buffer, 2, -Infinity);
}, "new DataView(buffer, 2, -Infinity);");
