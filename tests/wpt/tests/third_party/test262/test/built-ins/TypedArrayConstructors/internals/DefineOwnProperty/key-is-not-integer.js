// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns false if numericIndex is not an integer
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      i. If IsInteger(numericIndex) is false, return false.
  ...
includes: [testTypedArray.js]
features: [Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  assert.sameValue(
    Reflect.defineProperty(sample, "0.1", {
      value: 42,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "0.1"
  );
  assert.sameValue(sample[0], 0, "'0.1' - does not change the value for [0]");
  assert.sameValue(
    sample["0.1"],
    undefined,
    "'0.1' - does not define a value for ['0.1']"
  );

  assert.sameValue(
    Reflect.defineProperty(sample, "0.000001", {
      value: 42,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "0.000001"
  );
  assert.sameValue(
    sample[0], 0,
    "'0.000001' - does not change the value for [0]"
  );
  assert.sameValue(
    sample["0.000001"],
    undefined,
    "'0.000001' - does not define a value for ['0.000001']"
  );

  assert.sameValue(
    Reflect.defineProperty(sample, "1.1", {
      value: 42,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "1.1"
  );
  assert.sameValue(sample[1], 0, "'1.1' - does not change the value for [1]");
  assert.sameValue(
    sample["1.1"],
    undefined,
    "'1.1' - does not define a value for ['1.1']"
  );

  assert.sameValue(
    Reflect.defineProperty(sample, "Infinity", {
      value: 42,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "Infinity"
  );
  assert.sameValue(
    sample[0], 0,
    "'Infinity' - does not change the value for [0]"
  );
  assert.sameValue(
    sample[1], 0,
    "'Infinity' - does not change the value for [1]"
  );
  assert.sameValue(
    sample["Infinity"],
    undefined,
    "'Infinity' - does not define a value for ['Infinity']"
  );

  assert.sameValue(
    Reflect.defineProperty(sample, "-Infinity", {
      value: 42,
      configurable: false,
      enumerable: true,
      writable: true
    }),
    false,
    "-Infinity"
  );
  assert.sameValue(
    sample[0], 0,
    "'-Infinity' - does not change the value for [0]"
  );
  assert.sameValue(
    sample[1], 0,
    "'-Infinity' - does not change the value for [1]"
  );
  assert.sameValue(
    sample["-Infinity"],
    undefined,
    "'-Infinity' - does not define a value for ['-Infinity']"
  );

}, null, ["passthrough"]);
