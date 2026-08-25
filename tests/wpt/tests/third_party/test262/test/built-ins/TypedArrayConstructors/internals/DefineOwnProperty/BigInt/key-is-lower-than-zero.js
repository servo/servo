// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns false if numericIndex is < 0
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
      ii. Let intIndex be numericIndex.
      iv. If intIndex < 0, return false.
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  assert.sameValue(
    Reflect.defineProperty(sample, "-1", {
      value: 42n,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "-1"
  );
}, null, ["passthrough"]);
