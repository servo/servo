// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Return abrupt completion from GetMethod(searchValue.@@replace)
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    a. Let isRegExp be ? IsRegExp(searchString).
    b. If isRegExp is true, then
      i. Let flags be ? Get(searchValue, "flags").
      ii. Perform ? RequireObjectCoercible(flags).
      iii. If ? ToString(flags) does not contain "g", throw a TypeError exception.
    c. Let replacer be ? GetMethod(searchValue, @@replace).
  ...

  GetMethod ( V, P )

  ...
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
  4. If IsCallable(func) is false, throw a TypeError exception.
  5. Return func. 
features: [String.prototype.replaceAll, Symbol, Symbol.match, Symbol.replace]
---*/

var poisoned = 0;
var poison = {
  toString() {
    poisoned += 1;
    throw 'Should not call toString on this/replaceValue';
  },
};

var searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  get [Symbol.replace]() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, 'custom abrupt');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: {},
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is an object (not callable)');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: '',
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is a string');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: 42,
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is a number');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: Symbol(),
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is a symbol');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: true,
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is true');

searchValue = {
  [Symbol.match]: false,
  flags: 'g',
  [Symbol.replace]: false,
  toString() {
    throw 'Should not call toString on searchValue';
  }
};

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, '@@replace is false');

assert.sameValue(poisoned, 0);
