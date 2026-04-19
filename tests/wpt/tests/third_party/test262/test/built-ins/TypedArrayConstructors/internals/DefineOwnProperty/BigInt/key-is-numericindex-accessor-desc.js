// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-defineownproperty-p-desc
description: >
  Returns false if key is a numeric index and Descriptor is an
  AccessorDescriptor
info: |
  9.4.5.3 [[DefineOwnProperty]] ( P, Desc)
  ...
  3. If Type(P) is String, then
    a. Let numericIndex be ! CanonicalNumericIndexString(P).
    b. If numericIndex is not undefined, then
      ...
      vii. If IsAccessorDescriptor(Desc) is true, return false.
  ...
includes: [testTypedArray.js]
features: [BigInt, Reflect, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  assert.sameValue(
    Reflect.defineProperty(sample, "0", {
      get: function() {
        return 42;
      },
      enumerable: true
    }),
    false,
    "get accessor"
  );
  assert.sameValue(sample[0], 0n, "get accessor - side effect check");

  assert.sameValue(
    Reflect.defineProperty(sample, "0", {
      set: function() {},
      enumerable: true
    }),
    false,
    "set accessor"
  );
  assert.sameValue(sample[0], 0n, "set accessor - side effect check");

  assert.sameValue(
    Reflect.defineProperty(sample, "0", {
      set: function() {},
      get: function() {
        return 42;
      },
      enumerable: true
    }),
    false,
    "get and set accessors"
  );
  assert.sameValue(sample[0], 0n, "get and set accessors - side effect check");
}, null, ["passthrough"]);
