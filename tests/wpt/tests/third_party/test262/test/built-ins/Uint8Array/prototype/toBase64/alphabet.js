// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tobase64
description: Conversion of Uint8Arrays to base64 strings exercising the alphabet option
features: [uint8array-base64, TypedArray]
---*/

assert.sameValue((new Uint8Array([199, 239, 242])).toBase64(), "x+/y");

assert.sameValue((new Uint8Array([199, 239, 242])).toBase64({ alphabet: 'base64' }), "x+/y");

assert.sameValue((new Uint8Array([199, 239, 242])).toBase64({ alphabet: 'base64url' }), "x-_y");

assert.throws(TypeError, function() {
  (new Uint8Array([199, 239, 242])).toBase64({ alphabet: 'other' });
});
