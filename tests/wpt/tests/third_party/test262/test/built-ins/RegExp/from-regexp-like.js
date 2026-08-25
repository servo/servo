// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Initialization from a RegExp-like object
es6id: 21.2.3.1
info: |
    1. Let patternIsRegExp be IsRegExp(pattern).
    [...]
    6. Else if patternIsRegExp is true, then
       a. Let P be Get(pattern, "source").
       b. ReturnIfAbrupt(P).
       c. If flags is undefined, then
          i. Let F be Get(pattern, "flags").
          ii. ReturnIfAbrupt(F).
       d. Else, let F be flags.
    [...]
    10. Return RegExpInitialize(O, P, F).
features: [Symbol, Symbol.match]
---*/

var obj = {
  source: 'source text',
  flags: 'i'
};
var result;

obj[Symbol.match] = true;
result = new RegExp(obj);
assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.source, 'source text');
assert.sameValue(result.flags, 'i');

obj[Symbol.match] = 'string';
result = new RegExp(obj);
assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.source, 'source text');
assert.sameValue(result.flags, 'i');

obj[Symbol.match] = [];
result = new RegExp(obj);
assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.source, 'source text');
assert.sameValue(result.flags, 'i');

obj[Symbol.match] = Symbol();
result = new RegExp(obj);
assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.source, 'source text');
assert.sameValue(result.flags, 'i');

obj[Symbol.match] = 86;
result = new RegExp(obj);
assert.sameValue(Object.getPrototypeOf(result), RegExp.prototype);
assert.sameValue(result.source, 'source text');
assert.sameValue(result.flags, 'i');
