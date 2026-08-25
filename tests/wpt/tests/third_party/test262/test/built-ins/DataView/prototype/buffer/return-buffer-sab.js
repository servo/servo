// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.buffer
description: >
  Return buffer from [[ViewedArrayBuffer]] internal slot
info: |
  24.2.4.1 get DataView.prototype.buffer

  ...
  5. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  6. Return buffer.
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(1);
var dv = new DataView(buffer, 0);

assert.sameValue(dv.buffer, buffer);
