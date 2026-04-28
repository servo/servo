// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: >
    Arguments of mapping function (traversed via iterator)
info: |
    [...]
    2. If mapfn is undefined, let mapping be false.
    3. else
       a. If IsCallable(mapfn) is false, throw a TypeError exception.
       b. If thisArg was supplied, let T be thisArg; else let T be undefined.
       c. Let mapping be true
    [...]
    6. If usingIterator is not undefined, then
       [...]
       g. Repeat
          [...]
          vii. If mapping is true, then
               1. Let mappedValue be Call(mapfn, T, «nextValue, k»).
               2. If mappedValue is an abrupt completion, return
                  IteratorClose(iterator, mappedValue).
               3. Let mappedValue be mappedValue.[[value]].
features: [Symbol.iterator]
---*/

var args = [];
var firstResult = {
  done: false,
  value: {}
};
var secondResult = {
  done: false,
  value: {}
};
var mapFn = function(value, idx) {
  args.push(arguments);
};
var items = {};
var nextResult = firstResult;
var nextNextResult = secondResult;

items[Symbol.iterator] = function() {
  return {
    next: function() {
      var result = nextResult;
      nextResult = nextNextResult;
      nextNextResult = {
        done: true
      };

      return result;
    }
  };
};

Array.from(items, mapFn);

assert.sameValue(args.length, 2, 'The value of args.length is expected to be 2');

assert.sameValue(args[0].length, 2, 'The value of args[0].length is expected to be 2');
assert.sameValue(
  args[0][0], firstResult.value, 'The value of args[0][0] is expected to equal the value of firstResult.value'
);
assert.sameValue(args[0][1], 0, 'The value of args[0][1] is expected to be 0');

assert.sameValue(args[1].length, 2, 'The value of args[1].length is expected to be 2');
assert.sameValue(
  args[1][0], secondResult.value, 'The value of args[1][0] is expected to equal the value of secondResult.value'
);
assert.sameValue(args[1][1], 1, 'The value of args[1][1] is expected to be 1');
