// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Throws a TypeError if flags does not contain "g"
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
features: [String.prototype.replaceAll, Symbol.match, Symbol]
---*/

assert.sameValue(
  typeof String.prototype.replaceAll,
  'function',
  'function must exist'
);

var poisoned = 0;
var poison = {
  toString() {
    poisoned += 1;
    throw 'Should not call toString on this/replaceValue';
  },
};

var searchValue = /./;

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, 'flags is the empty string');

Object.defineProperty(searchValue, 'flags', {
  value: 'G',
  writable: true,
  configurable: true,
});

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, 'case sensitive G');

Object.defineProperty(searchValue, 'flags', {
  value: 'i',
  writable: true,
  configurable: true,
});

assert.throws(TypeError, function() {
  ''.replaceAll.call(poison, searchValue, poison);
}, 'i');

assert.sameValue(poisoned, 0);
