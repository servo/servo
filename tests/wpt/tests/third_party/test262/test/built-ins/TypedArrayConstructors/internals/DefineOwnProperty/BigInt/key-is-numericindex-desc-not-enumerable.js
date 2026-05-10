// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns false if key is a numeric index and Desc.[[Enumerable]] is false
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
      ix. If Desc has an [[Enumerable]] field and if Desc.[[Enumerable]] is
      false, return false.
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  assert.sameValue(
    Reflect.defineProperty(sample, "0", {
      value: 42n,
      configurable: false,
      enumerable: false,
      writable: true
    }),
    false,
    "defineProperty's result"
  );
  assert.sameValue(sample[0], 0n, "side effect check");
}, null, ["passthrough"]);
