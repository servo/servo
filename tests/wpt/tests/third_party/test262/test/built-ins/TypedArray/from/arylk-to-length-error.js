// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: Returns error produced by interpreting length property as a length
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  7. Let len be ? ToLength(? Get(arrayLike, "length")).
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var arrayLike = { length: {} };

arrayLike.length = {
  valueOf: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  TypedArray.from(arrayLike);
});
