// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.of
description: >
  Return abrupt from object value
info: |
  22.2.2.2 %TypedArray%.of ( ...items )

  ...
  7. Repeat, while k < len
    ...
    c. Perform ? Set(newObj, Pk, kValue, true).
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var lastValue = false;

  var obj1 = {
    valueOf() {
      lastValue = "obj1";
      return 42n;
    }
  };
  var obj2 = {
    valueOf() {
      lastValue = "obj2";
      throw new Test262Error();
    }
  };

  assert.throws(Test262Error, function() {
    TA.of(obj1, obj2, obj1);
  });

  assert.sameValue(lastValue, "obj2");
}, null, ["passthrough"]);

