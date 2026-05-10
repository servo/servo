// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Index coercion returns a throw completion.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  4. Let relativeIndex be ? ToIntegerOrInfinity(index).
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js]
---*/

function MyError() {}

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta = new TA(makeCtorArg(1));

  var index = {
    valueOf() {
      throw new MyError();
    }
  };

  var value = {
    valueOf() {
      throw new Test262Error("Unexpected value coercion");
    }
  };

  assert.throws(MyError, function() {
    ta.with(index, value);
  });
});
