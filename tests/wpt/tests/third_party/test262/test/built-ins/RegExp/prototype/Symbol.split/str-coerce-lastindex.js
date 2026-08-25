// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Length coercion of `lastIndex` property of splitter after a match
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        [...]
        c. Let z be RegExpExec(splitter, S).
        [...]
        f. Else z is not null,
           i. Let e be ToLength(Get(splitter, "lastIndex")).
           [...]
features: [Symbol.split, Symbol.species]
---*/

var result;
var obj = {
  constructor: function() {}
};
var fakeRe = {
  set lastIndex(_) {},
  get lastIndex() {
    return {
      valueOf: function() {
        return 2.9;
      }
    };
  },
  exec: function() {
    return [];
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

result = RegExp.prototype[Symbol.split].call(obj, 'abcd');

assert(Array.isArray(result));
assert.sameValue(result.length, 2);
assert.sameValue(result[0], '');
assert.sameValue(result[1], 'cd');
