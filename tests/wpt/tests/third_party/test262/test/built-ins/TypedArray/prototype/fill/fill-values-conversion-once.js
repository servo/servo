// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Fills all the elements with non numeric values values.
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  ...
  3. Let _value_ be ? ToNumber(_value_).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  var n = 1;
  sample.fill({ valueOf() { return n++; } });

  assert.sameValue(n, 2, "additional unexpected ToNumber() calls");
  assert.sameValue(sample[0], 1, "incorrect ToNumber result in index 0");
  assert.sameValue(sample[1], 1, "incorrect ToNumber result in index 1");
}, null, null, ["immutable"]);

