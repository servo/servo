// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tobase64
description: Conversion of Uint8Arrays to base64 strings
features: [uint8array-base64, TypedArray]
---*/

// standard test vectors from https://datatracker.ietf.org/doc/html/rfc4648#section-10
assert.sameValue((new Uint8Array([])).toBase64(), "");
assert.sameValue((new Uint8Array([102])).toBase64(), "Zg==");
assert.sameValue((new Uint8Array([102, 111])).toBase64(), "Zm8=");
assert.sameValue((new Uint8Array([102, 111, 111])).toBase64(), "Zm9v");
assert.sameValue((new Uint8Array([102, 111, 111, 98])).toBase64(), "Zm9vYg==");
assert.sameValue((new Uint8Array([102, 111, 111, 98, 97])).toBase64(), "Zm9vYmE=");
assert.sameValue((new Uint8Array([102, 111, 111, 98, 97, 114])).toBase64(), "Zm9vYmFy");
