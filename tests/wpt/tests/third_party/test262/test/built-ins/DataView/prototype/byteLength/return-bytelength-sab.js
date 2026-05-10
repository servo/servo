// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.bytelength
description: >
  Return value from [[ByteLength]] internal slot
info: |
  24.2.4.2 get DataView.prototype.byteLength

  ...
  7. Let size be the value of O's [[ByteLength]] internal slot.
  8. Return size.
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(12);

var sample1 = new DataView(buffer, 0);
var sample2 = new DataView(buffer, 4);
var sample3 = new DataView(buffer, 6, 4);
var sample4 = new DataView(buffer, 12);

assert.sameValue(sample1.byteLength, 12);
assert.sameValue(sample2.byteLength, 8);
assert.sameValue(sample3.byteLength, 4);
assert.sameValue(sample4.byteLength, 0);
