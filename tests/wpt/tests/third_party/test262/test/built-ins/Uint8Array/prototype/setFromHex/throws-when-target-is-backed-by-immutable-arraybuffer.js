// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-uint8array.prototype.setfromhex
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  Uint8Array.prototype.setFromHex ( string )
  1. Let into be the this value.
  2. Perform ? ValidateUint8Array(into).
features: [TypedArray, uint8array-base64, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg(["1", "2", "3", "4"]));

  assert.throws(TypeError, function() {
    ta.setFromHex("666f6f");
  }, "non-empty hex");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]),
    "Must not mutate contents (non-empty hex).");

  assert.throws(TypeError, function() {
    ta.setFromHex("");
  }, "empty hex");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]),
    "Must not mutate contents (empty hex).");
}, [Uint8Array], ["immutable"]);
