// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Conversion of base64 strings to Uint8Arrays
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

// standard test vectors from https://datatracker.ietf.org/doc/html/rfc4648#section-10
var standardBase64Vectors = [
  ["", []],
  ["Zg==", [102]],
  ["Zm8=", [102, 111]],
  ["Zm9v", [102, 111, 111]],
  ["Zm9vYg==", [102, 111, 111, 98]],
  ["Zm9vYmE=", [102, 111, 111, 98, 97]],
  ["Zm9vYmFy", [102, 111, 111, 98, 97, 114]],
];

standardBase64Vectors.forEach(function (pair) {
  var arr = Uint8Array.fromBase64(pair[0]);
  assert.sameValue(Object.getPrototypeOf(arr), Uint8Array.prototype, "decoding " + pair[0]);
  assert.sameValue(arr.length, pair[1].length, "decoding " + pair[0]);
  assert.sameValue(arr.buffer.byteLength, pair[1].length, "decoding " + pair[0]);
  assert.compareArray(arr, pair[1], "decoding " + pair[0]);
});
