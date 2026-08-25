// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Initialization from a RegExp-like object (with flag overrides)
es6id: 21.2.3.1
info: |
    1. Let patternIsRegExp be IsRegExp(pattern).
    [...]
    6. Else if patternIsRegExp is true, then
       a. Let P be Get(pattern, "source").
       b. ReturnIfAbrupt(P).
       c. If flags is undefined, then
          [...]
       d. Else, let F be flags.
    [...]
    10. Return RegExpInitialize(O, P, F).
features: [Symbol, Symbol.match]
---*/

var obj = {
  source: 'source text'
};
var result;

Object.defineProperty(obj, 'flags', {
  get: function() {
    throw new Test262Error('The `flags` property value should not be referenced.');
  }
});

obj[Symbol.match] = true;
result = new RegExp(obj, 'g');
assert.sameValue(
  result.source, 'source text', '@@match specified as a primitive boolean'
);
assert.sameValue(
  result.flags, 'g', '@@match specified as a primitive boolean'
);

obj[Symbol.match] = 'string';
result = new RegExp(obj, 'g');
assert.sameValue(
  result.source, 'source text', '@@match specified as a primitive string'
);
assert.sameValue(result.flags, 'g', '@@match specified as a primitive string');

obj[Symbol.match] = [];
result = new RegExp(obj, 'g');
assert.sameValue(
  result.source, 'source text', '@@match specified as an array'
);
assert.sameValue(result.flags, 'g', '@@match specified as an array');

obj[Symbol.match] = Symbol();
result = new RegExp(obj, 'g');
assert.sameValue(
  result.source, 'source text', '@@match specified as a Symbol'
);
assert.sameValue(result.flags, 'g', '@@match specified as a Symbol');

obj[Symbol.match] = 86;
result = new RegExp(obj, 'g');
assert.sameValue(
  result.source, 'source text', '@@match specified as a primitive number'
);
assert.sameValue(result.flags, 'g', '@@match specified as a primitive number');
