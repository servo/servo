// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint8
description: >
  Throws a TypeError exception when the backing buffer is immutable
info: |
  DataView.prototype.setInt8 ( byteOffset, value )
  1. Let view be the this value.
  2. Return ? SetViewValue(view, byteOffset, true, ~uint8~, value).

  SetViewValue ( view, requestIndex, isLittleEndian, type, value )
  1. Perform ? RequireInternalSlot(view, [[DataView]]).
  2. Assert: view has a [[ViewedArrayBuffer]] internal slot.
  3. If IsImmutableBuffer(view.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
  4. Let getIndex be ? ToIndex(requestIndex).
  5. If IsBigIntElementType(type) is true, let numberValue be ? ToBigInt(value).
  6. Otherwise, let numberValue be ? ToNumber(value).
features: [DataView, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var iab = (new ArrayBuffer(8)).transferToImmutable();
var view = new DataView(iab);

var calls = [];
var byteOffset = {
  valueOf() {
    calls.push("byteOffset.valueOf");
    return 0;
  }
};
var value = {
  valueOf() {
    calls.push("value.valueOf");
    return "1";
  }
};

assert.sameValue(
  view.getInt8(byteOffset),
  (new DataView(new ArrayBuffer(8))).getInt8(byteOffset),
  "read an initial zero"
);
calls = [];
assert.throws(TypeError, function() {
  view.setInt8(byteOffset, value);
});
assert.compareArray(calls, [], "Must verify mutability before reading arguments.");
