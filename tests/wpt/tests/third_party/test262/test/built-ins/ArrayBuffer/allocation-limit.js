// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  Throws a RangeError if requested Data Block is too large.
info: |
  ArrayBuffer( length )

  ...
  6. Return AllocateArrayBuffer(NewTarget, byteLength).

  6.2.6.1 CreateByteDataBlock(size)
    ...
    2. Let db be a new Data Block value consisting of size bytes. If it is
       impossible to create such a Data Block, throw a RangeError exception.
    ...
---*/

assert.throws(RangeError, function() {
  // Allocating 7 PiB should fail with a RangeError.
  // Math.pow(1024, 5) = 1125899906842624
  new ArrayBuffer(7 * 1125899906842624);
}, "`length` parameter is 7 PiB");

assert.throws(RangeError, function() {
  // Allocating almost 8 PiB should fail with a RangeError.
  // Math.pow(2, 53) = 9007199254740992
  new ArrayBuffer(9007199254740992 - 1);
}, "`length` parameter is Math.pow(2, 53) - 1");
