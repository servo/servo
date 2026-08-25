// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Error creating iterator object
info: |
    [...]
    6. If usingIterator is not undefined, then
       [...]
       d. Let iterator be GetIterator(items, usingIterator).
       e. ReturnIfAbrupt(iterator).
features: [Symbol.iterator]
---*/

var itemsPoisonedSymbolIterator = {};
itemsPoisonedSymbolIterator[Symbol.iterator] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Array.from(itemsPoisonedSymbolIterator);
}, 'Array.from(itemsPoisonedSymbolIterator) throws a Test262Error exception');
