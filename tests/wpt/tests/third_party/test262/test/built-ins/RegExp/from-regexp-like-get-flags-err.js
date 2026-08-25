// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error thrown from `flags` property of a RegExp-like object
es6id: 21.2.3.1
info: |
    1. Let patternIsRegExp be IsRegExp(pattern).
    [...]
    6. Else if patternIsRegExp is true, then
       [...]
       c. If flags is undefined, then
          i. Let F be Get(pattern, "flags").
          ii. ReturnIfAbrupt(F).
features: [Symbol, Symbol.match]
---*/

var obj = {};
Object.defineProperty(obj, 'flags', {
  get: function() {
    throw new Test262Error();
  }
});

obj[Symbol.match] = true;
assert.throws(Test262Error, function() {
  new RegExp(obj);
});

obj[Symbol.match] = 'string';
assert.throws(Test262Error, function() {
  new RegExp(obj);
});

obj[Symbol.match] = [];
assert.throws(Test262Error, function() {
  new RegExp(obj);
});

obj[Symbol.match] = Symbol();
assert.throws(Test262Error, function() {
  new RegExp(obj);
});

obj[Symbol.match] = 86;
assert.throws(Test262Error, function() {
  new RegExp(obj);
});
