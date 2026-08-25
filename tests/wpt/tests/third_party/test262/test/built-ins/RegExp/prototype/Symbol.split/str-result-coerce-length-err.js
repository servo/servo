// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    Behavior when error thrown while coercing `length` property of match result
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
               8. ReturnIfAbrupt(numberOfCaptures).
features: [Symbol.split, Symbol.species]
---*/

var obj = {
  constructor: function() {}
};
var uncoercibleLength;
var fakeRe = {
  exec: function() {
    return {
      length: uncoercibleLength
    };
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

uncoercibleLength = Symbol.split;
assert.throws(TypeError, function() {
  RegExp.prototype[Symbol.split].call(obj, 'abcd');
});

uncoercibleLength = {
  valueOf: function() {
    throw new Test262Error();
  }
};
assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.split].call(obj, 'abcd');
});
