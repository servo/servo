// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: PutValue operates only on references (see step 3.a).
es5id: 11.13.1-1-6-s
description: >
    simple assignment throws ReferenceError if LeftHandSide is an
    unresolvable reference (base obj undefined)
---*/

  
assert.throws(ReferenceError, function() {
    __ES3_1_test_suite_test_11_13_1_unique_id_0__.x = 42;
});
