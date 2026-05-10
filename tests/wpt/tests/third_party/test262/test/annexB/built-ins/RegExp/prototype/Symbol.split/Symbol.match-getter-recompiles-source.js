// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.prototype-@@split
description: >
  Side-effects in IsRegExp may recompile the regular expression.
info: |
  21.2.5.11 RegExp.prototype [ @@split ] ( string, limit )
    ...
    4. Let C be ? SpeciesConstructor(rx, %RegExp%).
    ...
    10. Let splitter be ? Construct(C, « rx, newFlags »).
    ...

  21.2.3.1 RegExp ( pattern, flags )
    1. Let patternIsRegExp be ? IsRegExp(pattern).
    ...

features: [Symbol.match, Symbol.split]
---*/

var regExp = /a/;
Object.defineProperty(regExp, Symbol.match, {
  get: function() {
    regExp.compile("b");
  }
});

var result = regExp[Symbol.split]("abba");

assert.sameValue(result.length, 3);
assert.sameValue(result[0], "a");
assert.sameValue(result[1], "");
assert.sameValue(result[2], "a");
