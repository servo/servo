// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-uint8array.prototype.setfrombase64
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  Uint8Array.prototype.setFromBase64 ( string [ , options ] )
  1. Let into be the this value.
  2. Perform ? ValidateUint8Array(into).
  3. If string is not a String, throw a TypeError exception.
  4. Let opts be ? GetOptionsObject(options).
  5. Let alphabet be ? Get(opts, "alphabet").
  6. If alphabet is undefined, set alphabet to "base64".
  7. If alphabet is neither "base64" nor "base64url", throw a TypeError exception.
  8. Let lastChunkHandling be ? Get(opts, "lastChunkHandling").
features: [TypedArray, uint8array-base64, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];

  var ta = new TA(makeCtorArg(["1", "2", "3", "4"]));
  var options = {
    get alphabet() {
      calls.push("get options.alphabet");
      return undefined;
    },
    get lastChunkHandling() {
      calls.push("get options.lastChunkHandling");
      return undefined;
    },
  };

  assert.throws(TypeError, function() {
    ta.setFromBase64("Zm9v", options);
  }, "non-empty base64");
  assert.compareArray(calls, [],
    "Must verify mutability before reading arguments (non-empty base64).");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]),
    "Must not mutate contents (non-empty base64).");

  calls = [];
  assert.throws(TypeError, function() {
    ta.setFromBase64("", options);
  }, "empty base64");
  assert.compareArray(calls, [],
    "Must verify mutability before reading arguments (empty base64).");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]),
    "Must not mutate contents (empty base64).");
}, [Uint8Array], ["immutable"]);
