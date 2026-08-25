// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Behavior when accessing `constructor` property of RegExp-like objects
es6id: 21.2.3.1
info: |
    1. Let patternIsRegExp be IsRegExp(pattern).
    [...]
    3. If NewTarget is not undefined, let newTarget be NewTarget.
    4. Else,
       a. Let newTarget be the active function object.
       b. If patternIsRegExp is true and flags is undefined, then
          i. Let patternConstructor be Get(pattern, "constructor").
          ii. ReturnIfAbrupt(patternConstructor).
          iii. If SameValue(newTarget, patternConstructor) is true, return
               pattern.
features: [Symbol, Symbol.match]
---*/

var obj = Object.defineProperty({}, 'constructor', {
  get: function() {
    throw new Test262Error();
  }
});

obj[Symbol.match] = true;
assert.throws(Test262Error, function() {
  RegExp(obj);
});

obj[Symbol.match] = 'string';
assert.throws(Test262Error, function() {
  RegExp(obj);
});

obj[Symbol.match] = [];
assert.throws(Test262Error, function() {
  RegExp(obj);
});

obj[Symbol.match] = Symbol()
assert.throws(Test262Error, function() {
  RegExp(obj);
});

obj[Symbol.match] = 86;
assert.throws(Test262Error, function() {
  RegExp(obj);
});
