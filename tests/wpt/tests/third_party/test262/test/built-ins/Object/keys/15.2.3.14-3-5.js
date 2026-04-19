// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-3-5
description: Object.keys must return a fresh array on each invocation
---*/

var literal = {
  a: 1
};
var keysBefore = Object.keys(literal);
assert.sameValue(keysBefore[0], 'a', 'keysBefore[0]');
keysBefore[0] = 'x';
var keysAfter = Object.keys(literal);

assert.sameValue(keysBefore[0], 'x', 'keysBefore[0]');
assert.sameValue(keysAfter[0], 'a', 'keysAfter[0]');
