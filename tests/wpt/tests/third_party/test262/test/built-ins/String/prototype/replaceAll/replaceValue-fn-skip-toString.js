// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  Skip ToString(replaceValue) if it's a function
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  1. Let O be RequireObjectCoercible(this value).
  2. If searchValue is neither undefined nor null, then
    ...
  3. Let string be ? ToString(O).
  4. Let searchString be ? ToString(searchValue).
  5. Let functionalReplace be IsCallable(replaceValue).
  6. If functionalReplace is false, then
    a. Let replaceValue be ? ToString(replaceValue). 
  ...
features: [String.prototype.replaceAll]
---*/

var called = 0;
var replaceValue = function() {
  called += 1;
  return 'b';
};
var poisoned = 0;
Object.defineProperty(replaceValue, 'toString', {
  value: function() {
    poisoned += 1;
    throw 'should not call this';
  }
});

var result = 'aaa'.replaceAll('a', replaceValue);
assert.sameValue(called, 3);
assert.sameValue(poisoned, 0);
assert.sameValue(result, 'bbb');
