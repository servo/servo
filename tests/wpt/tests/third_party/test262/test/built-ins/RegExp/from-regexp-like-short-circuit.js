// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Skipping construction from RegExp-like objects
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

var obj = {
  constructor: RegExp
};

obj[Symbol.match] = true;
assert.sameValue(RegExp(obj), obj);

obj[Symbol.match] = 'string';
assert.sameValue(RegExp(obj), obj);

obj[Symbol.match] = [];
assert.sameValue(RegExp(obj), obj);

obj[Symbol.match] = Symbol();
assert.sameValue(RegExp(obj), obj);

obj[Symbol.match] = 86;
assert.sameValue(RegExp(obj), obj);
