// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors when calling constructor's @@species
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    [...]
    2. If ? IsRegExp(R) is true, then
      a. Let C be ? SpeciesConstructor(R, RegExp).
      b. Let flags be ? ToString(? Get(R, "flags"))
      c. Let matcher be ? Construct(C, R, flags).
features: [Symbol.matchAll, Symbol.species]
---*/

var regexp = /./;
regexp.constructor = {
  [Symbol.species]: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  regexp[Symbol.matchAll]('');
});
