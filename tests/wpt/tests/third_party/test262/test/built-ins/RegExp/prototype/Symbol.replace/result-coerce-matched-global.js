// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  String coercion of "0" property of the value returned by RegExpExec.
  (global RegExp)
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
          b. Let nextIndex be AdvanceStringIndex(S, thisIndex, fullUnicode).
          c. Perform ? Set(rx, "lastIndex", nextIndex, true).
features: [Symbol.replace]
---*/

var r = /./g;
var coercibleValueWasReturned = false;
var coercibleValue = {
  length: 1,
  0: {
    toString: function() {
      return '';
    },
    valueOf: function() {
      throw new Test262Error('This method should not be invoked.');
    },
  },
  index: 0,
};

r.exec = function() {
  if (coercibleValueWasReturned) {
    return null;
  }

  coercibleValueWasReturned = true;
  return coercibleValue;
};

assert.sameValue(r[Symbol.replace]('', 'foo'), 'foo');
assert.sameValue(r.lastIndex, 1);
