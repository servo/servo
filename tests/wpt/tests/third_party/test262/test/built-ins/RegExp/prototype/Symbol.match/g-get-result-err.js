// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error is thrown while accessing `0` property of match result
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          [...]
          iv. Else result is not null,
              1. Let matchStr be ToString(Get(result, "0")).
              2. ReturnIfAbrupt(matchStr).
features: [Symbol.match]
---*/

var r = /./g;
var poisonedZero = {
  get 0() {
    throw new Test262Error();
  }
};

r.exec = function() {
  return poisonedZero;
};

assert.throws(Test262Error, function() {
  r[Symbol.match]('');
});
