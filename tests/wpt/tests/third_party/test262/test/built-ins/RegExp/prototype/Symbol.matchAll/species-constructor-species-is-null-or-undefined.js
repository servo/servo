// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: |
  Default constructor is used when species constructor is null or undefined
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    2. Return ? [MatchAllIterator](#matchalliterator)(R, string).

  MatchAllIterator ( R, O )
    [...]
    3. Let C be ? [SpeciesConstructor][species-constructor](R, RegExp).

  SpeciesConstructor ( O, defaultConstructor )
    [...]
    2. Let C be ? Get(O, "constructor").
    3. If C is undefined, return defaultConstructor.
    4. If Type(C) is not Object, throw a TypeError exception.
    5. Let S be ? Get(C, @@species).
    6. If S is either undefined or null, return defaultConstructor.
features: [Symbol.matchAll, Symbol.species]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

function TestWithConstructor(ctor) {
  var regexp = /\w/g;
  regexp.constructor = {
    [Symbol.species]: ctor
  };
  var str = 'a*b';

  assert.compareIterator(regexp[Symbol.matchAll](str), [
    matchValidator(['a'], 0, str),
    matchValidator(['b'], 2, str)
  ]);
}

TestWithConstructor(undefined);
TestWithConstructor(null);
