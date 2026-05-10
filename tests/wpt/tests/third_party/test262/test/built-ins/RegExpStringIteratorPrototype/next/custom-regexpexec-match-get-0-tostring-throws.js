// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown from coercing first match to a string
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    9. Let match be ? RegExpExec(R, S).
    10. If match is null, then
      [...]
    11. Else,
      a. If global is true,
        i. Let matchStr be ? ToString(? Get(match, "0")).
features: [Symbol.matchAll]
---*/

var iter = /./g[Symbol.matchAll]('');

RegExp.prototype.exec = function() {
  return [{
    toString: function() {
      throw new Test262Error();
    }
  }];
};

assert.throws(Test262Error, function() {
  iter.next();
});

