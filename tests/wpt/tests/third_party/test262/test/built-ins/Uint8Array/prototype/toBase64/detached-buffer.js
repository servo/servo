// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tobase64
description: Uint8Array.prototype.toBase64 checks for detachedness after side-effects are finished
includes: [detachArrayBuffer.js]
features: [uint8array-base64, TypedArray]
---*/

var array = new Uint8Array(2);
var getterCalls = 0;
var receiverDetachingOptions = {};
Object.defineProperty(receiverDetachingOptions, "alphabet", {
  get: function() {
    getterCalls += 1;
    $DETACHBUFFER(array.buffer);
    return "base64";
  }
});
assert.throws(TypeError, function() {
  array.toBase64(receiverDetachingOptions);
});
assert.sameValue(getterCalls, 1);


var detached = new Uint8Array(2);
$DETACHBUFFER(detached.buffer);
var getterCalls = 0;
var sideEffectingOptions = {};
Object.defineProperty(sideEffectingOptions, "alphabet", {
  get: function() {
    getterCalls += 1;
    return "base64";
  }
});
assert.throws(TypeError, function() {
  detached.toBase64(sideEffectingOptions);
});
assert.sameValue(getterCalls, 1);
