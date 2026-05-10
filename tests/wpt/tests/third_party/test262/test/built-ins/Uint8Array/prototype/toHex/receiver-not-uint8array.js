// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tohex
description: Uint8Array.prototype.toHex throws if the receiver is not a Uint8Array
includes: [testTypedArray.js]
features: [uint8array-base64, TypedArray]
---*/

var toHex = Uint8Array.prototype.toHex;

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  if (TA === Uint8Array) return;
  var sample = new TA(makeCtorArg(2));
  assert.throws(TypeError, function() {
    Uint8Array.prototype.toHex.call(sample);
  });
});

assert.throws(TypeError, function() {
  Uint8Array.prototype.toHex.call([]);
});

assert.throws(TypeError, function() {
  toHex();
});
