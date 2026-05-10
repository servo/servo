// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 7.4.3
description: >
    The `done` value of iteration result objects should be interpreted as
    incomplete as per `ToBoolean` (7.1.2).
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

firstIterResult = { value: null, done: undefined };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: null };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: false };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: true };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);

firstIterResult = { value: null, done: 1 };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);

firstIterResult = { value: null, done: 0 };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: -0 };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: NaN };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: '' };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 1);

firstIterResult = { value: null, done: '0' };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);

firstIterResult = { value: null, done: Symbol() };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);

firstIterResult = { value: null, done: {} };
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);

firstIterResult = { value: null };
Object.defineProperty(firstIterResult, 'done', {
  get: function() {
    return true;
  }
});
i = 0;
for (var x of iterable) {
  i++;
}
assert.sameValue(i, 0);
