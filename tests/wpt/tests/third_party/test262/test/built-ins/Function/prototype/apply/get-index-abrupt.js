// Copyright 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.apply
description: >
  Return abrupt completion from Get(obj, indexName)
info: |
  Function.prototype.apply ( thisArg, argArray )

  [...]
  4. Let argList be ? CreateListFromArrayLike(argArray).

  CreateListFromArrayLike ( obj [ , elementTypes ] )

  [...]
  6. Repeat, while index < len
    a. Let indexName be ! ToString(index).
    b. Let next be ? Get(obj, indexName).
---*/

var arrayLike = {
  length: 2,
  0: 0,
  get 1() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  (function() {}).apply(null, arrayLike);
});
