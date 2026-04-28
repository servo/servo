// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T8
description: >
    RegExp is /[a-z]/ and tested string is {toString:void 0,
    valueOf:function(){throw "invalof";}}
---*/

try {
  throw new Test262Error('#1.1: /[a-z]/.test({toString:void 0, valueOf:function(){throw "invalof";}}) throw "invalof". Actual: ' + (/[a-z]/.test({toString:void 0, valueOf:function(){throw "invalof";}})));
} catch (e) {
  assert.sameValue(e, "invalof", 'The value of e is expected to be "invalof"');
}

// TODO: Convert to assert.throws() format.
