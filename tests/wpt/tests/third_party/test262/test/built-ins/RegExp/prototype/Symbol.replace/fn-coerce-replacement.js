// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@replace
description: >
  String coercion of the value returned by functional replaceValue.
info: |
  RegExp.prototype [ @@replace ] ( string, replaceValue )

  [...]
  14. For each result in results, do
    [...]
    k. If functionalReplace is true, then
      [...]
      v. Let replValue be ? Call(replaceValue, undefined, replacerArgs).
      vi. Let replacement be ? ToString(replValue).
features: [Symbol.replace]
---*/

var replacer = function() {
  return {
    toString: function() {
      return 'toString value';
    },
    valueOf: function() {
      throw new Test262Error('This method should not be invoked.');
    },
  };
};

assert.sameValue(/x/[Symbol.replace]('[x]', replacer), '[toString value]');
