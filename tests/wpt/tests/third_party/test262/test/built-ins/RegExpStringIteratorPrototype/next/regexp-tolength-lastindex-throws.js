// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown coercing RegExp's lastIndex to a length
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    9. Let match be ? RegExpExec(R, S).
    10. If match is null, then
      [...]
    11. Else,
      a. If global is true,
        i. Let matchStr be ? ToString(? Get(match, "0")).
        ii. If matchStr is the empty string,
          1. Let thisIndex be ? ToLength(? Get(R, "lastIndex").
features: [Symbol.matchAll]
---*/

var iter = /./g[Symbol.matchAll]('');

RegExp.prototype.exec = function() {
  this.lastIndex = {
    valueOf() {
      throw new Test262Error();
    }
  };
  return [''];
};

assert.throws(Test262Error, function() {
  iter.next();
});
