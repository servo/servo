// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when first match is coerced to a empty string
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
          2. Let nextIndex be ! AdvanceStringIndex(S, thisIndex, fullUnicode).
          3. Perform ? Set(R, "lastIndex", nextIndex, true).
        iii. Return ! CreateIterResultObject(match, false).
features: [Symbol.matchAll]
---*/

var iter = /./g[Symbol.matchAll]('');

var execResult = {
  get '0'() {
    return {
      toString() { return ''; }
    };
  }
};

var internalRegExp;
RegExp.prototype.exec = function () {
  internalRegExp = this;
  return execResult;
};

var result = iter.next();
assert.sameValue(internalRegExp.lastIndex, 1);
assert.sameValue(result.value, execResult);
assert(!result.done);


result = iter.next();
assert.sameValue(internalRegExp.lastIndex, 2);
assert.sameValue(result.value, execResult);
assert(!result.done);
