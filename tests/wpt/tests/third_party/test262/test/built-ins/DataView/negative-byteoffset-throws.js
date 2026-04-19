// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a RangeError if ToInteger(byteOffset) < 0
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  4. Let offset be ? ToIndex(byteOffset).
  ...

  ToIndex ( value )

  1. If value is undefined, then
    a. Let index be 0.
  2. Else,
    a. Let integerIndex be ? ToInteger(value).
    b. If integerIndex < 0, throw a RangeError exception.
    ...
---*/

var buffer = new ArrayBuffer(2);

assert.throws(RangeError, function() {
  new DataView(buffer, -1);
}, "new DataView(buffer, -1);");

assert.throws(RangeError, function() {
  new DataView(buffer, -Infinity);
}, "new DataView(buffer, -Infinity);");
