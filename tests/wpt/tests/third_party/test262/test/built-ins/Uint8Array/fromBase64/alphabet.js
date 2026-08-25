// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Conversion of base64 strings to Uint8Arrays exercising the alphabet option
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

assert.compareArray(Uint8Array.fromBase64('x+/y'), [199, 239, 242]);
assert.compareArray(Uint8Array.fromBase64('x+/y', { alphabet: 'base64' }), [199, 239, 242]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('x+/y', { alphabet: 'base64url' });
});

assert.compareArray(Uint8Array.fromBase64('x-_y', { alphabet: 'base64url' }), [199, 239, 242]);
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('x-_y');
});
assert.throws(SyntaxError, function() {
  Uint8Array.fromBase64('x-_y', { alphabet: 'base64' });
});
