// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.13 S5.f
description: >
    The `value` of iteration result objects should be retrieved using the Get
    abstract operation.
features: [Symbol.iterator]
---*/

var iterable = {};
var i, firstIterResult;

iterable[Symbol.iterator] = function() {
  var finalIterResult = { value: null, done: true };
  var nextIterResult = firstIterResult;

  return {
    next: function() {
      var iterResult = nextIterResult;

      nextIterResult = finalIterResult;

      return iterResult;
    }
  };
};

firstIterResult = { value: 45, done: false };
i = 0;
for (var x of iterable) {
  assert.sameValue(x, 45);
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { done: false };
Object.defineProperty(firstIterResult, 'value', {
  get: function() {
    return 23;
  }
});
i = 0;
for (var x of iterable) {
  assert.sameValue(x, 23);
  i++;
}
assert.sameValue(i, 1);
