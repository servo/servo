// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    Behavior when error thrown while accessing capturing group match
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
               11. Repeat, while i ≤ numberOfCaptures.
                   [...]
                   a. Let nextCapture be Get(z, ToString(i)).
                   b. ReturnIfAbrupt(nextCapture).
features: [Symbol.split, Symbol.species]
---*/

var result;
var obj = {
  constructor: function() {}
};
var poisonedCapture = {
  length: 3,
  0: 'a',
  1: 'b',
  get 2() {
    throw new Test262Error();
  }
};
var fakeRe = {
  exec: function() {
    fakeRe.lastIndex = 1;
    return poisonedCapture;
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.split].call(obj, 'a');
});
