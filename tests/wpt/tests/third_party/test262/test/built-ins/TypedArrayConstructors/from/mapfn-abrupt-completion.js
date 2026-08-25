// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Return abrupt from mapfn
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  10. Repeat, while k < len
    ...
    c. If mapping is true, then
      i. Let mappedValue be ? Call(mapfn, T, « kValue, k »).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var source = {
  "0": 42,
  length: 2
};
var mapfn = function() {
  throw new Test262Error();
};

testWithTypedArrayConstructors(function(TA) {
  assert.throws(Test262Error, function() {
    TA.from(source, mapfn);
  });
}, null, ["passthrough"]);
