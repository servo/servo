// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error is thrown while coercing `0` property of match result
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
r.exec = function() {
  return {
    0: {
      toString: function() {
        throw new Test262Error();
      }
    }
  };
};

assert.throws(Test262Error, function() {
  r[Symbol.match]('');
});
