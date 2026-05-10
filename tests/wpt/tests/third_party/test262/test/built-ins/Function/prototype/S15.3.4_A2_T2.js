// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The Function prototype object is itself a Function object that, when
    invoked, accepts any arguments and returns undefined
es5id: 15.3.4_A2_T2
description: Call Function.prototype(null,void 0)
---*/

try {
  assert.sameValue(Function.prototype(null, void 0), undefined, 'Function.prototype(null, void 0) returns undefined');
} catch (e) {
  throw new Test262Error('#1.1: The Function prototype object is itself a Function object that, when invoked, accepts any arguments and returns undefined: ' + e);
}
