// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegExp.prototype.exec(string) Performs a regular expression match of ToString(string) against the regular expression and
    returns an Array object containing the results of the match, or null if the string did not match
es5id: 15.10.6.2_A1_T8
description: >
    String is {toString:void 0, valueOf:function(){throw "invalof";}}
    and RegExp is /[a-z]/
---*/

try {
  throw new Test262Error('#1.1: /[a-z]/.exec({toString:void 0, valueOf:function(){throw "invalof"}}) throw "invalof". Actual: ' + (/[a-z]/.exec({toString:void 0, valueOf:function(){throw "invalof"}})));
} catch (e) {
  assert.sameValue(e, "invalof", 'The value of e is expected to be "invalof"');
}

// TODO: Convert to assert.throws() format.
