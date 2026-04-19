// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  The `length` parameter can be zero.
info: |
  ArrayBuffer( length )

  ...
  2. Let numberLength be ToNumber(length).
  3. Let byteLength be ToLength(numberLength).
  4. ReturnIfAbrupt(byteLength).
  5. If SameValueZero(numberLength, byteLength) is false, throw a RangeError exception.
  ...
---*/

var positiveZero = new ArrayBuffer(+0);
assert.sameValue(positiveZero.byteLength, 0);

var negativeZero = new ArrayBuffer(-0);
assert.sameValue(negativeZero.byteLength, 0);
