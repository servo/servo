// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.fromhex
description: Uint8Array.fromHex throws a SyntaxError when input has non-hex characters
features: [uint8array-base64, TypedArray]
---*/

var illegal = [
  'a.a',
  'aa^',
  'a a',
  'a\ta',
  'a\x0Aa',
  'a\x0Ca',
  'a\x0Da',
  'a\u00A0a', // nbsp
  'a\u2009a', // thin space
  'a\u2028a', // line separator
];
illegal.forEach(function(value) {
  assert.throws(SyntaxError, function() {
    Uint8Array.fromHex(value)
  });
});
