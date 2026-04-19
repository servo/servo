// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Throws TypeError if `constructor` property is not an object
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
    3. If C is undefined, return defaultConstructor.
    4. If Type(C) is not Object, throw a TypeError exception.
features: [Symbol.matchAll]
---*/

var regexp = /./;

function callMatchAll() { regexp[Symbol.matchAll](''); }

regexp.constructor = null;
assert.throws(TypeError, callMatchAll, "`constructor` value is null");

regexp.constructor = true;
assert.throws(TypeError, callMatchAll, "`constructor` value is Boolean");

regexp.constructor = "";
assert.throws(TypeError, callMatchAll, "`constructor` value is String");

regexp.constructor = Symbol();
assert.throws(TypeError, callMatchAll, "`constructor` value is Symbol");

regexp.constructor = 1;
assert.throws(TypeError, callMatchAll, "`constructor` value is Number");
