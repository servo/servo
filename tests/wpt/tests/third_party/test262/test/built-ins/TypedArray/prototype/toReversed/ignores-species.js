// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.toreversed
description: >
  %TypedArray%.prototype.toReversed ignores @@species
info: |
  %TypedArray%.prototype.toReversed ( )

  ...
  4. Let A be ? TypedArrayCreateSameType(O, « 𝔽(length) »).
  ...

  TypedArrayCreateSameType ( exemplar, argumentList )
  ...
  2. Let constructor be the intrinsic object listed in column one of Table 63 for exemplar.[[TypedArrayName]].
  ...
includes: [testTypedArray.js]
features: [Symbol.species, TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg(0));
  ta.constructor = TA === Uint8Array ? Int32Array : Uint8Array;
  assert.sameValue(Object.getPrototypeOf(ta.toReversed()), TA.prototype);

  ta = new TA(makeCtorArg(0));
  ta.constructor = {
    [Symbol.species]: TA === Uint8Array ? Int32Array : Uint8Array,
  };
  assert.sameValue(Object.getPrototypeOf(ta.toReversed()), TA.prototype);

  ta = new TA(makeCtorArg(0));
  Object.defineProperty(ta, "constructor", {
    get() {
      throw new Test262Error("Should not get .constructor");
    }
  });
  ta.toReversed();
});
