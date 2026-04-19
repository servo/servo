// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: >
    Behavior when error thrown while setting `lastIndex` property of splitter
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        b. ReturnIfAbrupt(setStatus).
features: [Symbol.split, Symbol.species]
---*/

var callCount = 0;
var obj = {
  constructor: function() {}
};
obj.constructor[Symbol.species] = function() {
  return {
    set lastIndex(_) {
      throw new Test262Error();
    },
    exec: function() {
      callCount += 1;
    }
  };
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.split].call(obj, 'a');
});

assert.sameValue(callCount, 0);
