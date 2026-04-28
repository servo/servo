// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Value returned by mapping function (traversed via iterator)
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

var thisVals = [];
var nextResult = {
  done: false,
  value: {}
};
var nextNextResult = {
  done: false,
  value: {}
};
var firstReturnVal = {};
var secondReturnVal = {};
var mapFn = function(value, idx) {
  var returnVal = nextReturnVal;
  nextReturnVal = nextNextReturnVal;
  nextNextReturnVal = null;
  return returnVal;
};
var nextReturnVal = firstReturnVal;
var nextNextReturnVal = secondReturnVal;
var items = {};
var result;

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

result = Array.from(items, mapFn);

assert.sameValue(result.length, 2, 'The value of result.length is expected to be 2');
assert.sameValue(result[0], firstReturnVal, 'The value of result[0] is expected to equal the value of firstReturnVal');
assert.sameValue(
  result[1],
  secondReturnVal,
  'The value of result[1] is expected to equal the value of secondReturnVal'
);
