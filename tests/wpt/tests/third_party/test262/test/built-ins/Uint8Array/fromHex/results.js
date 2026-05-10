// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.fromhex
description: Conversion of hex strings to Uint8Arrays
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

var cases = [
  ["", []],
  ["66", [102]],
  ["666f", [102, 111]],
  ["666F", [102, 111]],
  ["666f6f", [102, 111, 111]],
  ["666F6f", [102, 111, 111]],
  ["666f6f62", [102, 111, 111, 98]],
  ["666f6f6261", [102, 111, 111, 98, 97]],
  ["666f6f626172", [102, 111, 111, 98, 97, 114]],
];

cases.forEach(function (pair) {
  var arr = Uint8Array.fromHex(pair[0]);
  assert.sameValue(Object.getPrototypeOf(arr), Uint8Array.prototype, "decoding " + pair[0]);
  assert.sameValue(arr.length, pair[1].length, "decoding " + pair[0]);
  assert.sameValue(arr.buffer.byteLength, pair[1].length, "decoding " + pair[0]);
  assert.compareArray(arr, pair[1], "decoding " + pair[0]);
});
