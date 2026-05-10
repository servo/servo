// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: let P be ToString(pattern) and let F be ToString(flags)
es5id: 15.10.4.1_A8_T7
description: >
    Pattern is {toString:void 0, valueOf:function(){throw "invalof";}
    } and flags is "i"
---*/

try {
    throw new Test262Error('#1.1: new RegExp({toString:void 0, valueOf:function(){throw "invalof";}}) throw "invalof". Actual: ' + (new RegExp({toString:void 0, valueOf:function(){throw "invalof";}})));
} catch (e) {
  assert.sameValue(e, "invalof", 'The value of e is expected to be "invalof"');
}

// TODO: Convert to assert.throws() format.
