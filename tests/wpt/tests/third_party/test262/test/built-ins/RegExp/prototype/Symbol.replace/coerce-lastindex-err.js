// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  Abrupt completion during coercion of "lastIndex" property of `this` value.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  11. Repeat, while done is false
    a. Let result be ? RegExpExec(rx, S).
    b. If result is null, set done to true.
    c. Else,
      i. Append result to the end of results.
      ii. If global is false, set done to true.
      iii. Else,
        1. Let matchStr be ? ToString(? Get(result, "0")).
        2. If matchStr is the empty String, then
          a. Let thisIndex be ? ToLength(? Get(rx, "lastIndex")).
features: [Symbol.replace]
---*/

var r = /./g;
var execWasCalled = false;
var coercibleIndex = {
  valueOf: function() {
    throw new Test262Error();
  },
};

var result = {
  length: 1,
  0: '',
  index: 0,
};

r.exec = function() {
  if (execWasCalled) {
    return null;
  }

  r.lastIndex = coercibleIndex;
  execWasCalled = true;
  return result;
};

assert.throws(Test262Error, function() {
  r[Symbol.replace]('', '');
});
