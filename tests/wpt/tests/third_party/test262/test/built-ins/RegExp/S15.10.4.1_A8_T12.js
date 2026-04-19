// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: let P be ToString(pattern) and let F be ToString(flags)
es5id: 15.10.4.1_A8_T12
description: >
    Pattern is "\u0042" and flags is {toString:void 0,
    valueOf:function(){throw "invalof";} }
---*/

try {
    throw new Test262Error('#1.1: new RegExp("\\u0042", {toString:void 0, valueOf:function(){throw "invalof";}}) throw "invalof". Actual: ' + (new RegExp("\u0042", {toString:void 0, valueOf:function(){throw "invalof";}})));
} catch (e) {
  assert.sameValue(e, "invalof", 'The value of e is expected to be "invalof"');
}

// TODO: Convert to assert.throws() format.
