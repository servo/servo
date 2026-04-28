// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Setting length of object (traversed via iterator)
info: |
    [...]
    6. If usingIterator is not undefined, then
       [...]
       g. Repeat
          [...]
          iv. If next is false, then
              1. Let setStatus be Set(A, "length", k, true).
              2. ReturnIfAbrupt(setStatus).
              3. Return A.
features: [Symbol.iterator]
---*/

var items = {};
var result, nextIterResult, lastIterResult;
items[Symbol.iterator] = function() {
  return {
    next: function() {
      var result = nextIterResult;
      nextIterResult = lastIterResult;
      return result;
    }
  };
};

nextIterResult = lastIterResult = {
  done: true
};
result = Array.from(items);

assert.sameValue(result.length, 0, 'The value of result.length is expected to be 0');

nextIterResult = {
  done: false
};
lastIterResult = {
  done: true
};
result = Array.from(items);

assert.sameValue(result.length, 1, 'The value of result.length is expected to be 1');
