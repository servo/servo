// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Object prototype object has not prototype
es5id: 15.2.4_A1_T2
description: >
    Since the Object prototype object has not prototype, deleted
    toString method can not be found in prototype chain
---*/
assert.notSameValue(Object.prototype.toString(), false, 'Object.prototype.toString() must return false');

delete Object.prototype.toString;

try {
  Object.prototype.toString();
  throw new Test262Error('#2: Object prototype object has not prototype');
} catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}
//
