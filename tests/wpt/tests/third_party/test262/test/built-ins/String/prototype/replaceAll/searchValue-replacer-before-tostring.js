// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  The searchValue is observed before ToString(this value) and ToString(replaceValue)
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
    d. If replacer is not undefined, then
      i. Return ? Call(replacer, searchValue, « O, replaceValue »).
  3. Let string be ? ToString(O).
  4. Let searchString be ? ToString(searchValue).
  ...
features: [String.prototype.replaceAll, Symbol.replace]
---*/

var called = 0;
var searchValue = /./g;
Object.defineProperty(searchValue, Symbol.replace, {
  value: function(O, replaceValue) {
    assert.sameValue(this, searchValue);
    assert.sameValue(O, poison, 'first arg is the this value of replaceAll');
    assert.sameValue(replaceValue, poison, 'second arg is the replaceValue');
    assert.sameValue(arguments.length, 2);
    called += 1;
    return 'return from searchValue';
  }
});

Object.defineProperty(searchValue, 'toString', {
  value: function() {
    throw 'Should not call toString on searchValue';
  }
});

var poisoned = 0;
var poison = {
  toString() {
    poisoned += 1;
    throw 'Should not call toString on this/replaceValue';
  },
};

var returned = ''.replaceAll.call(poison, searchValue, poison);

assert.sameValue(returned, 'return from searchValue');
assert.sameValue(called, 1);
assert.sameValue(poisoned, 0);
