// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Return Abrupt completion from isRegExp
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    a. Let isRegExp be ? IsRegExp(searchString).
  ...

  IsRegExp ( argument )

  1. If Type(argument) is not Object, return false.
  2. Let matcher be ? Get(argument, @@match).
  3. If matcher is not undefined, return ! ToBoolean(matcher).
  4. If argument has a [[RegExpMatcher]] internal slot, return true.
  5. Return false. 
features: [String.prototype.replaceAll, Symbol.match]
---*/

var searchValue = {
  get [Symbol.match]() {
    throw new Test262Error();
  },
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

var poisoned = 0;
var poison = {
  toString() {
    poisoned += 1;
    throw 'Should not call toString on this/replaceValue';
  },
};

assert.throws(Test262Error, function() {
  ''.replaceAll.call(poison, searchValue, poison);
});

assert.sameValue(poisoned, 0);
