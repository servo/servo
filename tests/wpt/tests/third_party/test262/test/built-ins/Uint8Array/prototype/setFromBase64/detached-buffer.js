// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfrombase64
description: Uint8Array.prototype.setFromBase64 throws on detatched buffers
includes: [detachArrayBuffer.js]
features: [uint8array-base64, TypedArray]
---*/

var target = new Uint8Array([255, 255, 255]);
$DETACHBUFFER(target.buffer);
assert.throws(TypeError, function() {
  target.setFromBase64('Zg==');
});

var getterCalls = 0;
var targetDetachingOptions = {};
Object.defineProperty(targetDetachingOptions, 'alphabet', {
  get: function() {
    getterCalls += 1;
    $DETACHBUFFER(target.buffer);
    return "base64";
  }
});
var target = new Uint8Array([255, 255, 255]);
assert.throws(TypeError, function() {
  target.setFromBase64('Zg==', targetDetachingOptions);
});
assert.sameValue(getterCalls, 1);
