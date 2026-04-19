// Copyright 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.apply
description: >
  Return abrupt completion from Get(obj, "length")
info: |
  Function.prototype.apply ( thisArg, argArray )

  [...]
  4. Let argList be ? CreateListFromArrayLike(argArray).

  CreateListFromArrayLike ( obj [ , elementTypes ] )

  [...]
  3. Let len be ? ToLength(? Get(obj, "length")).
---*/

var arrayLike = {
  get length() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  (function() {}).apply(null, arrayLike);
});
