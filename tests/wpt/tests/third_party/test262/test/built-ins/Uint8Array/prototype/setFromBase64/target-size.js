// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfrombase64
description: Uint8Array.prototype.setFromBase64 behavior when target buffer is small
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

// buffer too small
var target = new Uint8Array([255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmFy');
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [102, 111, 111, 255, 255]);

// buffer too small, padded
var target = new Uint8Array([255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmE=');
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [102, 111, 111, 255]);

// buffer exact
var target = new Uint8Array([255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmFy');
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 6);
assert.compareArray(target, [102, 111, 111, 98, 97, 114]);

// buffer exact, padded
var target = new Uint8Array([255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmE=');
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 5);
assert.compareArray(target, [102, 111, 111, 98, 97]);

// buffer exact, not padded
var target = new Uint8Array([255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmE');
assert.sameValue(result.read, 7);
assert.sameValue(result.written, 5);
assert.compareArray(target, [102, 111, 111, 98, 97]);

// buffer exact, padded, stop-before-partial
var target = new Uint8Array([255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmE=', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 5);
assert.compareArray(target, [102, 111, 111, 98, 97]);

// buffer exact, not padded, stop-before-partial
var target = new Uint8Array([255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmE', { lastChunkHandling: 'stop-before-partial' });
assert.sameValue(result.read, 4);
assert.sameValue(result.written, 3);
assert.compareArray(target, [102, 111, 111, 255, 255]);

// buffer too large
var target = new Uint8Array([255, 255, 255, 255, 255, 255, 255]);
var result = target.setFromBase64('Zm9vYmFy');
assert.sameValue(result.read, 8);
assert.sameValue(result.written, 6);
assert.compareArray(target, [102, 111, 111, 98, 97, 114, 255]);
