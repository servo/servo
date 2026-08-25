// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-uint8array.prototype.setfromhex
description: >
  Throws a SyntaxError when the string length is odd.
info: |
  Uint8Array.prototype.setFromHex ( string )

  ...
  7. Let result be FromHex(string, byteLength).
  ...
  13. If result.[[Error]] is not none, then
    a. Throw result.[[Error]].
  ...

  FromHex ( string [ , maxLength ] )

  ...
  5. If length modulo 2 is not 0, then
    a. Let error be a new SyntaxError exception.
    b. Return the Record { [[Read]]: read, [[Bytes]]: bytes, [[Error]]: error }.
  ...

features: [TypedArray, uint8array-base64]
---*/

var zeroLength = new Uint8Array(0);

assert.throws(SyntaxError, function() {
  zeroLength.setFromHex("1")
}, "Uint8Array has length 0");


var nonZeroLength = new Uint8Array(1);

assert.throws(SyntaxError, function() {
  nonZeroLength.setFromHex("1")
}, "Uint8Array has length >0");
