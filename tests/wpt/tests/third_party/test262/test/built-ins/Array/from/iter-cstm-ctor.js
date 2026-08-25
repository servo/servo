// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Creating object with custom constructor (traversed via iterator)
info: |
    [...]
    6. If usingIterator is not undefined, then
       a. If IsConstructor(C) is true, then
          i. Let A be Construct(C).
       b. Else,
          i. Let A be ArrayCreate(0).
       c. ReturnIfAbrupt(A).
features: [Symbol.iterator]
---*/

var thisVal, args;
var callCount = 0;
var C = function() {
  thisVal = this;
  args = arguments;
  callCount += 1;
};
var result;
var items = {};
items[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: true
      };
    }
  };
};

result = Array.from.call(C, items);

assert(
  result instanceof C, 'The result of evaluating (result instanceof C) is expected to be true'
);
assert.sameValue(
  result.constructor,
  C,
  'The value of result.constructor is expected to equal the value of C'
);
assert.sameValue(callCount, 1, 'The value of callCount is expected to be 1');
assert.sameValue(thisVal, result, 'The value of thisVal is expected to equal the value of result');
assert.sameValue(args.length, 0, 'The value of args.length is expected to be 0');
