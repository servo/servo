// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tobase64
description: Conversion of Uint8Arrays to base64 strings exercising the omitPadding option
features: [uint8array-base64, TypedArray]
---*/

// works with default alphabet
assert.sameValue((new Uint8Array([199, 239])).toBase64(), "x+8=");
assert.sameValue((new Uint8Array([199, 239])).toBase64({ omitPadding: false }), "x+8=");
assert.sameValue((new Uint8Array([199, 239])).toBase64({ omitPadding: true }), "x+8");
assert.sameValue((new Uint8Array([255])).toBase64({ omitPadding: true }), "/w");

// works with base64url alphabet
assert.sameValue((new Uint8Array([199, 239])).toBase64({ alphabet: "base64url" }), "x-8=");
assert.sameValue((new Uint8Array([199, 239])).toBase64({ alphabet: "base64url", omitPadding: false }), "x-8=");
assert.sameValue((new Uint8Array([199, 239])).toBase64({ alphabet: "base64url", omitPadding: true }), "x-8");
assert.sameValue((new Uint8Array([255])).toBase64({ alphabet: "base64url", omitPadding: true }), "_w");

// performs ToBoolean on the argument
assert.sameValue((new Uint8Array([255])).toBase64({ omitPadding: 0 }), "/w==");
assert.sameValue((new Uint8Array([255])).toBase64({ omitPadding: 1 }), "/w");
