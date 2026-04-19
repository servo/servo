// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when error thrown from `source` property of a RegExp-like object
es6id: 21.2.3.1
info: |
    1. Let patternIsRegExp be IsRegExp(pattern).
    [...]
    6. Else if patternIsRegExp is true, then
       a. Let P be Get(pattern, "source").
       b. ReturnIfAbrupt(P).
features: [Symbol, Symbol.match]
---*/

var obj = {};
function CustomError() {}
Object.defineProperty(obj, 'source', {
  get: function() {
    throw new CustomError();
  }
});
Object.defineProperty(obj, 'flags', {
  get: function() {
    throw new Test262Error('the `flags` property should not be referenced before `source`');
  }
});

obj[Symbol.match] = true;
assert.throws(CustomError, function() {
  new RegExp(obj);
});

obj[Symbol.match] = 'string';
assert.throws(CustomError, function() {
  new RegExp(obj);
});

obj[Symbol.match] = [];
assert.throws(CustomError, function() {
  new RegExp(obj);
});

obj[Symbol.match] = Symbol();
assert.throws(CustomError, function() {
  new RegExp(obj);
});

obj[Symbol.match] = 86;
assert.throws(CustomError, function() {
  new RegExp(obj);
});
