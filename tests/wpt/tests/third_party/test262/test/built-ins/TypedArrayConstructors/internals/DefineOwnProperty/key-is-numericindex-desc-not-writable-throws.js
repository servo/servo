// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Throws TypeError for valid index & non-writable descriptor.
info: |
  [[DefineOwnProperty]] ( P, Desc )

  [...]
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      [...]
      v. If Desc has a [[Writable]] field and if Desc.[[Writable]] is false, return false.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([0]));

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", {
      writable: false,
    });
  }, "partial descriptor");

  assert.throws(TypeError, function() {
    Object.defineProperty(sample, "0", {
      value: 42,
      writable: false,
      enumerable: true,
      configurable: true,
    });
  }, "complete descriptor");

  assert.sameValue(sample[0], 0, "side effect check");
}, null, ["passthrough"]);
