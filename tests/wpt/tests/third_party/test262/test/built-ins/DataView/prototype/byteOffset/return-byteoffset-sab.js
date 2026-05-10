// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.byteoffset
description: >
  Return value from [[ByteOffset]] internal slot
info: |
  24.2.4.3 get DataView.prototype.byteOffset

  ...
  7. Let offset be the value of O's [[ByteOffset]] internal slot.
  8. Return offset.
features: [SharedArrayBuffer]
---*/

var buffer = new SharedArrayBuffer(12);

var sample1 = new DataView(buffer, 0);
var sample2 = new DataView(buffer, 4);
var sample3 = new DataView(buffer, 6, 4);
var sample4 = new DataView(buffer, 12);
var sample5 = new DataView(buffer, 0, 2);

assert.sameValue(sample1.byteOffset, 0);
assert.sameValue(sample2.byteOffset, 4);
assert.sameValue(sample3.byteOffset, 6);
assert.sameValue(sample4.byteOffset, 12);
assert.sameValue(sample5.byteOffset, 0);
