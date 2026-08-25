// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Setting `lastIndex` property of splitter after a match
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
           iv. Else e ≠ p,
                [...]
                6. Let p be e.
                [...]
                12. Let q be p.
features: [Symbol.split, Symbol.species]
---*/

var obj = {
  constructor: function() {}
};
var lastIndex = 0;
var indices = '';
var fakeRe = {
  set lastIndex(val) {
    lastIndex = val;
    indices += val + ',';
  },
  get lastIndex() {
    return lastIndex;
  },
  exec: function() {
    lastIndex += 1;
    return ['a'];
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

RegExp.prototype[Symbol.split].call(obj, 'abcd');

assert.sameValue(indices, '0,1,2,3,');
