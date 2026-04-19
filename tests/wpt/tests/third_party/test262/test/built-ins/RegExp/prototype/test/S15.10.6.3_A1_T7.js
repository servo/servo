// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Equivalent to the expression RegExp.prototype.exec(string) != null
es5id: 15.10.6.3_A1_T7
description: >
    RegExp is /[a-z]/ and tested string is {toString:function(){throw
    "intostr";}}
---*/

try {
    throw new Test262Error('#1.1: /[a-z]/.test({toString:function(){throw "intostr";}}) throw "intostr". Actual: ' + (/[a-z]/.test({toString:function(){throw "intostr";}})));
} catch (e) {
  assert.sameValue(e, "intostr", 'The value of e is expected to be "intostr"');
}

// TODO: Convert to assert.throws() format.
