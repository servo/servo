// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.setfromhex
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
  var allFF = [255, 255, 255, 255, 255, 255, 255, 255];
  var target = new Uint8Array(allFF);
  var result = target.setFromHex(pair[0]);
  assert.sameValue(result.read, pair[0].length);
  assert.sameValue(result.written, pair[1].length);

  var expected = pair[1].concat(allFF.slice(pair[1].length))
  assert.compareArray(target, expected, "decoding " + pair[0]);
});
