// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Return Abrupt completion from Get(searchValue, "flags")
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    a. Let isRegExp be ? IsRegExp(searchString).
    b. If isRegExp is true, then
      i. Let flags be ? Get(searchValue, "flags").
      ii. Perform ? RequireObjectCoercible(flags).
      iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
  ...
features: [String.prototype.replaceAll, Symbol.match]
---*/

var searchValue = {
  [Symbol.match]: true,
  get flags() {
    throw new Test262Error;
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
}, 'from custom searchValue object');

var re1 = /./;
Object.defineProperty(re1, 'flags', {
  get() { throw new Test262Error(); }
});

assert.throws(Test262Error, function() {
  ''.replaceAll.call(poison, re1, poison);
}, 'from RE instance, using default Symbol.match check');

var called = 0;
var re2 = /./;
Object.defineProperty(re2, Symbol.match, {
  get() {
    called += 1;
    return true;
  }
});
Object.defineProperty(re2, 'flags', {
  get() { throw new Test262Error(); }
});

assert.throws(Test262Error, function() {
  ''.replaceAll.call(poison, re2, poison);
}, 'from RE instance, using Symbol.match check (true)');
assert.sameValue(called, 1);

called = 0;
var re3 = /./;
Object.defineProperty(re3, Symbol.match, {
  get() {
    called += 1;
    return 1;
  }
});
Object.defineProperty(re3, 'flags', {
  get() { throw new Test262Error(); }
});

assert.throws(Test262Error, function() {
  ''.replaceAll.call(poison, re3, poison);
}, 'from RE instance, using Symbol.match check (1), uses Internal for IsRegExp');
assert.sameValue(called, 1);

assert.sameValue(poisoned, 0);
