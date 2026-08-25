// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error is thrown while accessing `1` property of result
es6id: 21.2.5.8
info: |
    [...]
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).
        [...]
    16. Repeat, for each result in results,
        [...]
        l. Repeat while n ≤ nCaptures
           i. Let capN be Get(result, ToString(n)).
           ii. ReturnIfAbrupt(capN).
features: [Symbol.replace]
---*/

var r = /./;
var poisonedValue = {
  length: 2,
  get 1() {
    throw new Test262Error();
  }
};
r.exec = function() {
  return poisonedValue;
};

assert.throws(Test262Error, function() {
  r[Symbol.replace]('a', 'b');
});
