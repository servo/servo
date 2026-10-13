// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Instance buffer can be detached during loop
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  9. Repeat, while k < len
    ...
    b. Let kValue be ? Get(O, Pk).
    c. Let selected be ToBoolean(? Call(callbackfn, T, « kValue, k, O »)).
  ...
includes: [detachArrayBuffer.js, testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var loops = 0;
  var sample = new TA(makeCtorArg(2));

  var result = sample.filter(function(value, index) {
    if (loops === 0) {
      assert.sameValue(value, 0);
      $DETACHBUFFER(sample.buffer);
    } else {
      assert.sameValue(value, undefined,
        "callback element is undefined for index " + index);
    }
    loops++;
    return true;
  });

  assert.sameValue(loops, 2);
  var converted = isFloatTypedArrayConstructor(TA) ? NaN : 0;
  assert.compareArray(result, [0, converted]);
}, null, null, ["immutable"]);
