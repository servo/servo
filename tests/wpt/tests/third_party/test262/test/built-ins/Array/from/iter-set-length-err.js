// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Error setting length of object (traversed via iterator)
info: |
    [...]
    6. If usingIterator is not undefined, then
       [...]
       g. Repeat
          [...]
          iv. If next is false, then
              1. Let setStatus be Set(A, "length", k, true).
              2. ReturnIfAbrupt(setStatus).
features: [Symbol.iterator]
---*/

var poisonedPrototypeLength = function() {};
var items = {};
Object.defineProperty(poisonedPrototypeLength.prototype, 'length', {
  set: function(_) {
    throw new Test262Error();
  }
});
items[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: true
      };
    }
  };
};

assert.throws(Test262Error, function() {
  Array.from.call(poisonedPrototypeLength, items);
}, 'Array.from.call(poisonedPrototypeLength, items) throws a Test262Error exception');
