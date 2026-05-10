// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error is thrown while type coercing `0` property of result
es6id: 21.2.5.8
info: |
    [...]
    13. Repeat, while done is false
        a. Let result be RegExpExec(rx, S).
        [...]
    16. Repeat, for each result in results,
        [...]
        d. Let matched be ToString(Get(result, "0")).
        e. ReturnIfAbrupt(matched).
features: [Symbol.replace]
---*/

var r = /./;
var uncoercibleValue = {
  0: {
    toString: function() {
      throw new Test262Error();
    }
  }
};
r.exec = function() {
  return uncoercibleValue;
};

assert.throws(Test262Error, function() {
  r[Symbol.replace]('a', 'b');
});
