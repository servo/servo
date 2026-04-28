// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfromhex
description: Uint8Array.prototype.setFromHex takes into account the offset of the target Uint8Array
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

var base = new Uint8Array([255, 255, 255, 255, 255, 255, 255]);
var subarray = base.subarray(2, 5);

var result = subarray.setFromHex('aabbcc');
assert.sameValue(result.read, 6);
assert.sameValue(result.written, 3);
assert.compareArray(subarray, [170, 187, 204]);
assert.compareArray(base, [255, 255, 170, 187, 204, 255, 255]);
