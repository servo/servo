// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: No matching attempt is made when `limit` argument is `0`
info: |
    [...]
    21. If lim = 0, return A.
features: [Symbol.split, Symbol.species]
---*/

var result;
var obj = {
  constructor: function() {}
};
obj.constructor[Symbol.species] = function() {
  return {
    exec: function() {
      throw new Test262Error('No match should be attempted when `limit` is `0`.');
    }
  };
};

result = RegExp.prototype[Symbol.split].call(obj, '', 0);

assert(Array.isArray(result));
assert.sameValue(result.length, 0);
