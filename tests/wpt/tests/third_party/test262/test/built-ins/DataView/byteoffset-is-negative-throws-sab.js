// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Throws a RangeError if ToInteger(byteOffset) < 0
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  4. Let numberOffset be ? ToNumber(byteOffset).
  5. Let offset be ToInteger(numberOffset).
  6. If numberOffset ≠ offset or offset < 0, throw a RangeError exception.
  ...
features: [SharedArrayBuffer]
---*/

var ab = new SharedArrayBuffer(42);

assert.throws(RangeError, function() {
  new DataView(ab, -1);
}, "-1");

assert.throws(RangeError, function() {
  new DataView(ab, -Infinity);
}, "-Infinity");
