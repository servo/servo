// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flatmap
description: >
  Observe abrupt completion in poisoned lengths of array-like objects
info: |
  Array.prototype.flatMap ( mapperFunction [ , thisArg ] )

  1. Let O be ? ToObject(this value).
  2. Let sourceLen be ? ToLength(? Get(O, "length")).
features: [Array.prototype.flatMap, Symbol.toPrimitive]
---*/

assert.sameValue(typeof Array.prototype.flatMap, 'function');

function fn(e) {
  return e;
}

var arr = {
  length: Symbol(),
};
assert.throws(TypeError, function() {
  [].flatMap.call(arr, fn);
}, 'length is a symbol');

arr = {
  get length() { throw new Test262Error() }
};
assert.throws(Test262Error, function() {
  [].flatMap.call(arr, fn);
}, 'custom get error');

arr = {
  length: {
    valueOf() { throw new Test262Error() }
  }
};
assert.throws(Test262Error, function() {
  [].flatMap.call(arr, fn);
}, 'custom valueOf error');

arr = {
  length: {
    toString() { throw new Test262Error() }
  }
};
assert.throws(Test262Error, function() {
  [].flatMap.call(arr, fn);
}, 'custom toString error');

arr = {
  length: {
    [Symbol.toPrimitive]() { throw new Test262Error() }
  }
};
assert.throws(Test262Error, function() {
  [].flatMap.call(arr, fn);
}, 'custom Symbol.toPrimitive error');
