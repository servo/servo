// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Throws TypeError for valid index & accessor descriptor.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      [...]
      iv. If IsAccessorDescriptor(Desc) is true, return false.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([0n]));

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", {
      get: function() { return 42n; },
    });
  }, "get accessor");
  assert.sameValue(sample[0], 0n, "get accessor - side effect check");

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", {
      set: function(_v) {},
    });
  }, "set accessor");
  assert.sameValue(sample[0], 0n, "set accessor - side effect check");

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", {
      get: function() { return 42n; },
      set: function(_v) {},
    });
  }, "get and set accessors");
  assert.sameValue(sample[0], 0n, "get and set accessors - side effect check");
}, null, ["passthrough"]);
