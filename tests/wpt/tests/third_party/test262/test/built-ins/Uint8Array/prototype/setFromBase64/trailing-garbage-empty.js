// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-uint8array.prototype.setfrombase64
description: >
  Garbage input is ignored when the typed array has a zero length
info: |
  Uint8Array.prototype.setFromBase64 ( string [ , options ] )
  ...
  13. Let byteLength be TypedArrayLength(taRecord).
  14. Let result be FromBase64(string, alphabet, lastChunkHandling, byteLength).
  ...

  FromBase64 ( string, alphabet, lastChunkHandling [ , maxLength ] )
  ...
  3. If maxLength = 0, then
    a. Return the Record { [[Read]]: 0, [[Bytes]]: « », [[Error]]: none }.
  ...

features: [uint8array-base64, TypedArray]
---*/

// Zero length typed array.
var u8 = new Uint8Array(0);

// No SyntaxError when passing invalid inputs.
for (var string of [
  "#",
  "a#",
  "aa#",
  "aaa#",
  "aaaa#",
]) {
  for (var lastChunkHandling of ["loose", "strict", "stop-before-partial"]) {
    var result = u8.setFromBase64(string, {lastChunkHandling});
    assert.sameValue(
      result.read,
      0,
      `Read for "${string}" with lastChunkHandling="${lastChunkHandling}"`
    );
    assert.sameValue(
      result.written,
      0,
      `Write for "${string}" with lastChunkHandling="${lastChunkHandling}"`
    );
  }
}
