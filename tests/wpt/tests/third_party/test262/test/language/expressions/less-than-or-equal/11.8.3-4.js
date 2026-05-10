// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.8.3-4
description: >
    11.8.3 Less-than-or-equal Operator - Partial left to right order
    enforced when using Less-than-or-equal operator: toString <=
    toString
---*/

var accessed = false;
var obj1 = {
  toString: function () {
    accessed = true;
    return 4;
  }
};
var obj2 = {
  toString: function () {
    return 2;
  }
};

assert.sameValue(obj1 <= obj2, false, 'The result of (obj1 <= obj2) is false');
assert.sameValue(accessed, true, 'The value of accessed is true');
