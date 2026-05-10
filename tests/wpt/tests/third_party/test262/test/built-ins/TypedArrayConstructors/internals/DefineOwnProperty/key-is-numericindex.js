// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns true after setting a valid numeric index key
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      If ! IsValidIntegerIndex(O, numericIndex) is false, return false.
      If IsAccessorDescriptor(Desc) is true, return false.
      If Desc has a [[Configurable]] field and if Desc.[[Configurable]] is false, return false.
      If Desc has an [[Enumerable]] field and if Desc.[[Enumerable]] is false, return false.
      If Desc has a [[Writable]] field and if Desc.[[Writable]] is false, return false.
      If Desc has a [[Value]] field, then
        Let value be Desc.[[Value]].
        Return ? IntegerIndexedElementSet(O, numericIndex, value).

includes: [testTypedArray.js]
features: [Reflect, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 42]));

  assert.sameValue(
    Reflect.defineProperty(sample, "0", {
      value: 8,
      configurable: true,
      enumerable: true,
      writable: true
    }),
    true
  );

  assert.sameValue(sample[0], 8, "property value was set");
  let descriptor0 = Object.getOwnPropertyDescriptor(sample, "0");
  assert.sameValue(descriptor0.value, 8);
  assert.sameValue(descriptor0.configurable, true);
  assert.sameValue(descriptor0.enumerable, true);
  assert.sameValue(descriptor0.writable, true);
}, null, ["passthrough"]);
