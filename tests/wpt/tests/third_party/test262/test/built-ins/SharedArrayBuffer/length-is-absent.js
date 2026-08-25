// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-sharedarraybuffer-length
description: >
  Returns an empty instance if length is absent
info: |
  SharedArrayBuffer( length )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. Let byteLength be ? ToIndex(length).
  3. Return ? AllocateSharedArrayBuffer(NewTarget, byteLength).
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer();

assert.sameValue(buffer.byteLength, 0);
