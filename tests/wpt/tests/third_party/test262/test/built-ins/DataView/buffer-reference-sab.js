// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview-buffer-byteoffset-bytelength
description: >
  Reuse buffer argument instead of making a new clone
info: |
  24.2.2.1 DataView (buffer, byteOffset, byteLength )

  ...
  14. Set O's [[ViewedArrayBuffer]] internal slot to buffer.
  ...
  17. Return O.
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(8);

var dv1 = new DataView(buffer, 0);
var dv2 = new DataView(buffer, 0);

assert.sameValue(dv1.buffer, buffer);
assert.sameValue(dv2.buffer, buffer);
