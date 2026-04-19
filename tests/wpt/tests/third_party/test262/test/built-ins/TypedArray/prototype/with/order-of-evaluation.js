// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Index parameter is coerced before value parameter.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  4. Let relativeIndex be ? ToIntegerOrInfinity(index).
  ...
  8. Else, let numericValue be ? ToNumber(value).
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg(1));

  var logs = [];

  var index = {
    valueOf() {
      logs.push("index");
      return 0;
    }
  };

  var value = {
    valueOf() {
      logs.push("value");
      return 0;
    }
  };

  ta.with(index, value);

  assert.compareArray(logs, ["index", "value"]);
});
