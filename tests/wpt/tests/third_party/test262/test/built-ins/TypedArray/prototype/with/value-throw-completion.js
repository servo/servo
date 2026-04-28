// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  Value coercion returns a throw completion.
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  7. If O.[[ContentType]] is bigint, let numericValue be ? ToBigInt(value).
  8. Else, let numericValue be ? ToNumber(value).
  ...
features: [TypedArray, change-array-by-copy]
includes: [testTypedArray.js]
---*/

function MyError() {}

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta = new TA(makeCtorArg(1));

  var value = {
    valueOf() {
      throw new MyError();
    }
  };

  assert.throws(MyError, function() {
    ta.with(100, value);
  }, "Positive too large index");

  assert.throws(MyError, function() {
    ta.with(-100, value);
  }, "Negative too large index");
});
