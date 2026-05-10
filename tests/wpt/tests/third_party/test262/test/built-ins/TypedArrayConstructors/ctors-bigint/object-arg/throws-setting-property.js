// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-object
description: >
  Return abrupt from setting property
info: |
  22.2.4.4 TypedArray ( object )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.

  ...
  8. Repeat, while k < len
    ...
    b. Let kValue be ? Get(arrayLike, Pk).
    c. Perform ? Set(O, Pk, kValue, true).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var obj = {
  "2": {
    valueOf() {
      throw new Test262Error();
    }
  },
  length: 4
};

testWithBigIntTypedArrayConstructors(function(TA) {
  obj[0] = 0n;
  obj[1] = 0n;
  assert.throws(Test262Error, function() {
    new TA(obj);
  });
}, null, ["passthrough"]);
