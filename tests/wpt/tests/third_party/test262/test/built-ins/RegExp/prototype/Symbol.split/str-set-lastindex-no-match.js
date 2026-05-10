// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Setting `lastIndex` property of splitter after a failed match
info: |
    [...]
    24. Repeat, while q < size
        a. Let setStatus be Set(splitter, "lastIndex", q, true).
        [...]
        e. If z is null, let q be AdvanceStringIndex(S, q, unicodeMatching).
        [...]
features: [Symbol.split, Symbol.species]
---*/

var obj = {
  constructor: function() {}
};
var indices = '';
var fakeRe = {
  set lastIndex(val) {
    indices += val + ',';
  },
  exec: function() {
    return null;
  }
};
obj.constructor[Symbol.species] = function() {
  return fakeRe;
};

RegExp.prototype[Symbol.split].call(obj, 'abcd');

assert.sameValue(indices, '0,1,2,3,');
