// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfrombase64
description: Uint8Array.prototype.setFromBase64 decodes and writes chunks which occur prior to bad data
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

var target = new Uint8Array([255, 255, 255, 255, 255]);
assert.throws(SyntaxError, function() {
  target.setFromBase64("MjYyZm.9v");
}, "illegal character in second chunk");
assert.compareArray(target, [50, 54, 50, 255, 255], "decoding from MjYyZm.9v should only write the valid chunks");

target = new Uint8Array([255, 255, 255, 255, 255]);
assert.throws(SyntaxError, function() {
  target.setFromBase64("MjYyZg", { lastChunkHandling: "strict" });
}, "padding omitted with lastChunkHandling: strict");
assert.compareArray(target, [50, 54, 50, 255, 255], "decoding from MjYyZg should only write the valid chunks");

target = new Uint8Array([255, 255, 255, 255, 255]);
assert.throws(SyntaxError, function() {
  target.setFromBase64("MjYyZg===");
}, "extra characters after padding");
assert.compareArray(target, [50, 54, 50, 255, 255], "decoding from MjYyZg=== should not write the last chunk because it has extra padding");
