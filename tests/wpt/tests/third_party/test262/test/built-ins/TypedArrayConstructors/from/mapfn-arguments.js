// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: >
  Assert mapfn arguments
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

var source = [42, 43, 44];

testWithTypedArrayConstructors(function(TA) {
  var results = [];
  var mapfn = function(kValue, k) {
    results.push({
      kValue: kValue,
      k: k,
      argsLength: arguments.length
    });
  };

  TA.from(source, mapfn);

  assert.sameValue(results.length, 3);

  assert.sameValue(results[0].kValue, 42);
  assert.sameValue(results[0].k, 0);
  assert.sameValue(results[0].argsLength, 2);

  assert.sameValue(results[1].kValue, 43);
  assert.sameValue(results[1].k, 1);
  assert.sameValue(results[1].argsLength, 2);

  assert.sameValue(results[2].kValue, 44);
  assert.sameValue(results[2].k, 2);
  assert.sameValue(results[2].argsLength, 2);
}, null, ["passthrough"]);
