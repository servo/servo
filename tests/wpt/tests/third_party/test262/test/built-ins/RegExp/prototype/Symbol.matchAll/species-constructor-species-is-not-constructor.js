// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: TypeError is thrown when species constructor is not a constructor
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
    7. If IsConstructor(S) is true, return S.
    8. Throw a TypeError exception.
features: [Symbol.matchAll, Symbol.species]
---*/

var regexp = /./;
var speciesConstructor = {};
regexp.constructor = speciesConstructor;

var callMatchAll = function() {
  regexp[Symbol.matchAll]('');
}

speciesConstructor[Symbol.species] = true;
assert.throws(TypeError, callMatchAll, "`constructor[Symbol.species]` value is Boolean");

speciesConstructor[Symbol.species] = 1;
assert.throws(TypeError, callMatchAll, "`constructor[Symbol.species]` value is Number");

speciesConstructor[Symbol.species] = Symbol();
assert.throws(TypeError, callMatchAll, "`constructor[Symbol.species]` value is Symbol");

speciesConstructor[Symbol.species] = true;
assert.throws(TypeError, callMatchAll, "`constructor[Symbol.species]` value is Boolean");
