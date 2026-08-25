// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.from
description: Setting property on result value (traversed via iterator)
info: |
    [...]
    6. If usingIterator is not undefined, then
       [...]
       g. Repeat
          [...]
          ix. Let defineStatus be CreateDataPropertyOrThrow(A, Pk,
              mappedValue).
features: [Symbol.iterator]
---*/

var items = {};
var firstIterResult = {
  done: false,
  value: {}
};
var secondIterResult = {
  done: false,
  value: {}
};
var thirdIterResult = {
  done: true,
  value: {}
};
var nextIterResult = firstIterResult;
var nextNextIterResult = secondIterResult;
var result;

items[Symbol.iterator] = function() {
  return {
    next: function() {
      var result = nextIterResult;

      nextIterResult = nextNextIterResult;
      nextNextIterResult = thirdIterResult;

      return result;
    }
  };
};

result = Array.from(items);

assert.sameValue(
  result[0],
  firstIterResult.value,
  'The value of result[0] is expected to equal the value of firstIterResult.value'
);
assert.sameValue(
  result[1],
  secondIterResult.value,
  'The value of result[1] is expected to equal the value of secondIterResult.value'
);
