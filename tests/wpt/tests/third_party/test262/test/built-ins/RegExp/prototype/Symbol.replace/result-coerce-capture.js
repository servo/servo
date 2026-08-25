// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  String coercion of "3" property of the value returned by RegExpExec.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  11. Repeat, while done is false
    a. Let result be ? RegExpExec(rx, S).
    [...]
  14. For each result in results, do
    [...]
    i. Repeat, while n ≤ nCaptures
      i. Let capN be ? Get(result, ! ToString(n)).
      ii. If capN is not undefined, then
        1. Set capN to ? ToString(capN).
        [...]
features: [Symbol.replace]
---*/

var r = /./;
var coercibleValue = {
  length: 4,
  index: 0,
  3: {
    toString: function() {
      return 'toString value';
    },
    valueOf: function() {
      throw new Test262Error('This method should not be invoked.');
    },
  },
};
r.exec = function() {
  return coercibleValue;
};

assert.sameValue(
  r[Symbol.replace]('', 'foo[$3]bar'), 'foo[toString value]bar'
);
