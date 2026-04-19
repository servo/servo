// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  Returns an empty instance if length is absent
info: |
  ArrayBuffer( length )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Return ? AllocateArrayBuffer(NewTarget, byteLength).
---*/

var buffer = new ArrayBuffer();

assert.sameValue(buffer.byteLength, 0);
