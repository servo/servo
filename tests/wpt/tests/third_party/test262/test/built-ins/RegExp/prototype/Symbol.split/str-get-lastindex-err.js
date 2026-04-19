// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    Behavior when error thrown while accessing `lastIndex` property of splitter
    after a match
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        [...]
        c. Let z be RegExpExec(splitter, S).
        [...]
        f. Else z is not null,
           i. Let e be ToLength(Get(splitter, "lastIndex")).
           ii. ReturnIfAbrupt(e).
features: [Symbol.split, Symbol.species]
---*/

var obj = {
  constructor: function() {}
};
var callCount = 0;
var fakeRe = {
  set lastIndex(_) {},
  get lastIndex() {
    throw new Test262Error();
  },
  exec: function() {
    callCount += 1;
    return [];
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.split].call(obj, 'abcd');
});

assert.sameValue(callCount, 1);
