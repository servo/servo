// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown while accessing of @@species property
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    [...]
    2. If ? IsRegExp(R) is true, then
      a. Let C be ? SpeciesConstructor(R, RegExp).

  SpeciesConstructor ( O, defaultConstructor )
    [...]
    2. Let C be ? Get(O, "constructor").
features: [Symbol.matchAll, Symbol.species]
---*/

var regexp = /./;
regexp.constructor = {
  get [Symbol.species]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  regexp[Symbol.matchAll]('');
});
