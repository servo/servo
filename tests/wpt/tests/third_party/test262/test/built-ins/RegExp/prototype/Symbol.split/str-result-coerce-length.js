// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    Length coercion of `length` property of match result
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        [...]
        c. Let z be RegExpExec(splitter, S).
        [...]
        f. Else z is not null,
           iv. Else e ≠ p,
               [...]
               7. Let numberOfCaptures be ToLength(Get(z, "length")).
               [...]
features: [Symbol.split, Symbol.species]
---*/

var result;
var obj = {
  constructor: function() {}
};
var fakeRe = {
  exec: function() {
    fakeRe.lastIndex = 1;
    return {
      length: {
        valueOf: function() {
          return 2.9;
        }
      },
      0: 'foo',
      1: 'bar',
      2: 'baz'
    };
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

result = RegExp.prototype[Symbol.split].call(obj, 'a');

assert(Array.isArray(result));
assert.sameValue(result.length, 3);
assert.sameValue(result[0], '');
assert.sameValue(result[1], 'bar');
assert.sameValue(result[2], '');
