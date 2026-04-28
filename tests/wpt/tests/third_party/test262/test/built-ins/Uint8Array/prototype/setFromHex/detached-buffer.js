// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfromhex
description: Uint8Array.prototype.setFromHex throws on detatched buffers
includes: [detachArrayBuffer.js]
features: [uint8array-base64, TypedArray]
---*/

var target = new Uint8Array([255, 255, 255]);
$DETACHBUFFER(target.buffer);
assert.throws(TypeError, function() {
  target.setFromHex('aa');
});
