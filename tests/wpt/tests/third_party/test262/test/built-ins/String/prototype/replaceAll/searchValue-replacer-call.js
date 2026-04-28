// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Return value from Call(replacer, ...)
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
  5. Let functionalReplace be IsCallable(replaceValue).
  6. If functionalReplace is false, then
    a. Let replaceValue be ? ToString(replaceValue). 
  ...
features: [String.prototype.replaceAll, Symbol.replace]
---*/

var called = 0;
var searchValue = /./g;
Object.defineProperty(searchValue, Symbol.replace, {
  value: function(O, replaceValue) {
    assert.sameValue(this, searchValue);
    assert.sameValue(O, str, 'first arg is the this value of replaceAll');
    assert.sameValue(replaceValue, obj, 'second arg is the replaceValue');
    assert.sameValue(arguments.length, 2);
    called += 1;
    return 42;
  }
});

Object.defineProperty(searchValue, 'toString', {
  value: function() {
    throw 'Should not call searchValue toString'
  }
});

var str = new String('Leo');
var obj = {};

var returned = str.replaceAll(searchValue, obj);

assert.sameValue(returned, 42);
assert.sameValue(called, 1);
