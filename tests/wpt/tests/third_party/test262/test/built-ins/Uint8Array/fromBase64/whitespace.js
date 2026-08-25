// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Uint8Array.fromBase64 ignores ASCII whitespace in the input
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

var whitespaceKinds = [
  ["Z g==", "space"],
  ["Z\tg==", "tab"],
  ["Z\x0Ag==", "LF"],
  ["Z\x0Cg==", "FF"],
  ["Z\x0Dg==", "CR"],
];
whitespaceKinds.forEach(function(pair) {
  var arr = Uint8Array.fromBase64(pair[0]);
  assert.sameValue(arr.length, 1);
  assert.sameValue(arr.buffer.byteLength, 1);
  assert.compareArray(arr, [102], "ascii whitespace: " + pair[1]);
});
